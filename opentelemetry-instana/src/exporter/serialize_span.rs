use anyhow::{anyhow, Result};
use opentelemetry::trace::{SpanId, SpanKind, Status, TraceId};
use opentelemetry::Value;
use opentelemetry_sdk::trace::SpanData;
use serde_json::json;
use std::collections::HashMap;
use std::time::SystemTime;

use crate::InstanaExporter;
use crate::exporter::instana_span::{
    InstanaCustom, InstanaEvent, InstanaLink, InstanaOtel, InstanaSdk, InstanaSpan,
    InstanaSpanData, InstanaSpanFrom, InstanaTags,
};
use crate::exporter::span_data::GET;

/// Convert an OpenTelemetry SpanData to an InstanaSpan
pub fn convert_to_instana_span(exporter: &InstanaExporter, span: &SpanData) -> Result<InstanaSpan> {
    let tid = span.span_context.trace_id();
    let trace_id = format!("{:032x}", tid);
    let span_id = format!("{:016x}", span.span_context.span_id());

    // Determine span kind value (1=entry, 2=exit, 3=intermediate)
    let kind_value = match span.span_kind {
        SpanKind::Server | SpanKind::Producer => 1,
        SpanKind::Internal => 3,
        SpanKind::Client | SpanKind::Consumer => 2,
    };

    // Get parent span ID if available
    let parent_id = if span.parent_span_id != SpanId::INVALID {
        Some(format!("{:016x}", span.parent_span_id))
    } else {
        None
    };

    // Get long trace ID for entry spans
    let long_trace_id = match span.span_kind {
        SpanKind::Server | SpanKind::Producer => {
            if tid != TraceId::INVALID {
                Some(trace_id.clone())
            } else {
                None
            }
        },
        _ => None,
    };

    // Check synthetic flag
    let synthetic = match span.get_attribute("X-INSTANA-SYNTHETIC") {
        Ok(value) => value == Value::I64(1),
        _ => false,
    };

    // Calculate timestamps and duration
    let start_time = span
        .start_time
        .duration_since(SystemTime::UNIX_EPOCH)?
        .as_millis() as u64;

    let duration = span
        .end_time
        .duration_since(span.start_time)
        .map_err(|e| anyhow!("Error parsing duration: {}", e))?
        .as_millis() as u64;

    // Error count
    let error_count = if span.status == Status::error("something went wrong") {
        Some(1)
    } else {
        None
    };

    // Process correlation data
    let (correlation_id, correlation_type) = extract_correlation_data(span)?;

    // Process trace parent flag
    let trace_parent = match span.get_attribute("INTERNAL_TAG_TP") {
        Ok(value) => {
            if let Value::Bool(val) = value {
                Some(val)
            } else {
                None
            }
        },
        _ => None,
    };

    // Build the data section
    let data = build_data_section(exporter, span)?;

    // Build the from section
    let from = build_from_section(exporter);

    // Create the InstanaSpan
    let instana_span = InstanaSpan {
        parent_span_id: parent_id,
        trace_id: trace_id[16..32].to_string(), // Use right-most 16 chars
        span_id,
        name: "sdk".to_string(),
        kind: kind_value,
        timestamp: start_time,
        duration,
        synthetic,
        long_trace_id,
        error_count,
        correlation_id,
        correlation_type,
        trace_parent,
        data,
        from,
    };

    Ok(instana_span)
}

/// Extract correlation ID and type from span
fn extract_correlation_data(span: &SpanData) -> Result<(Option<String>, Option<String>)> {
    if let Some(value) = span.span_context.trace_state().get("X-INSTANA-L") {
        let parts: Vec<&str> = value.split(',').map(|s| s.trim()).collect();
        let level_value = if let Some(value) = parts[0].split(':').last() {
            value.trim()
        } else {
            "0"
        };

        if level_value == "1" && parts.len() >= 2 {
            let correlation_values: Vec<&str> = parts[1].split(';').map(|s| s.trim()).collect();
            let mut correlation_type = None;
            let mut correlation_id = None;

            for part in correlation_values.iter() {
                if part.starts_with("correlationType=") {
                    correlation_type =
                        Some(part.trim_start_matches("correlationType=").to_string());
                } else if part.starts_with("correlationId=") {
                    correlation_id = Some(part.trim_start_matches("correlationId=").to_string());
                }
            }

            return Ok((correlation_id, correlation_type));
        }
    } else {
        let crid = match span.get_attribute("INTERNAL_TAG_CRID") {
            Ok(value) => Some(value.to_string()),
            _ => None,
        };

        let crtp = match span.get_attribute("INTERNAL_TAG_CRTP") {
            Ok(value) => Some(value.to_string()),
            _ => None,
        };

        return Ok((crid, crtp));
    }

    Ok((None, None))
}

/// Build the data section of the InstanaSpan
fn build_data_section(exporter: &InstanaExporter, span: &SpanData) -> Result<InstanaSpanData> {
    // Convert attributes to HashMap
    let attributes = if !span.attributes.is_empty() {
        let mut attrs = HashMap::new();
        for attr in span.get_attributes() {
            attrs.insert(attr.key.to_string(), convert_value_to_json(&attr.value));
        }
        Some(attrs)
    } else {
        None
    };

    // Convert resource attributes
    let resource = {
        let mut res_attrs = HashMap::new();
        for (key, value) in exporter.get_resource_attributes().iter() {
            let key_str = key.to_string();
            if !key_str.contains("service.") {
                res_attrs.insert(key_str, value.to_string());
            }
        }
        if res_attrs.is_empty() {
            None
        } else {
            Some(res_attrs)
        }
    };

    // Convert events
    let events = if !span.events.events.is_empty() {
        let mut events_map = HashMap::new();
        for event in span.get_events() {
            let mut attrs = HashMap::new();
            for attr in &event.attributes {
                attrs.insert(attr.key.to_string(), convert_value_to_json(&attr.value));
            }

            let timestamp = event
                .timestamp
                .duration_since(SystemTime::UNIX_EPOCH)?
                .as_millis()
                .to_string();

            events_map.insert(
                event.name.to_string(),
                InstanaEvent {
                    value: attrs,
                    timestamp,
                },
            );
        }
        Some(events_map)
    } else {
        None
    };

    // Convert links
    let links = if !span.links.links.is_empty() {
        let mut links_vec = Vec::new();
        for link in span.get_links() {
            let trace_id = format!("{:032x}", link.span_context.trace_id());
            let span_id = format!("{:016x}", link.span_context.span_id());

            let mut attrs = HashMap::new();
            for attr in &link.attributes {
                attrs.insert(attr.key.to_string(), convert_value_to_json(&attr.value));
            }

            links_vec.push(InstanaLink {
                trace_id: trace_id[16..32].to_string(),
                span_id,
                attributes: attrs,
            });
        }
        Some(links_vec)
    } else {
        None
    };

    // Calculate dropped counts
    let dropped_events_count = span.events.dropped_count;
    let dropped_links_count = span.links.dropped_count;
    let dropped_attributes_count = span.dropped_attributes_count
        + span
            .events
            .events
            .iter()
            .map(|e| e.dropped_attributes_count)
            .sum::<u32>()
        + span
            .links
            .links
            .iter()
            .map(|l| l.dropped_attributes_count)
            .sum::<u32>();

    // Process status
    let (status_code, status_description) = match &span.status {
        Status::Ok => (Some("StatusCode::STATUS_OK".to_string()), None),
        Status::Error { description } => {
            let desc = description.to_string();
            (Some("StatusCode::STATUS_ERROR".to_string()), Some(desc))
        },
        Status::Unset => (None, None),
    };

    // Build OTEL section
    let otel = InstanaOtel {
        scope_name: span.instrumentation_scope.name().to_string(),
        scope_version: span.instrumentation_scope.version().map(|v| v.to_string()),
        dropped_events_count,
        dropped_links_count,
        dropped_attributes_count,
        status_code,
        status_description,
    };

    // Build tags section
    let tags = InstanaTags {
        attributes,
        resource,
        events,
        links,
        otel,
    };

    // Build custom section
    let custom = InstanaCustom { tags };

    // Build SDK section
    let sdk = InstanaSdk {
        name: span.name.to_string(),
        span_type: convert_span_kind_to_string(&span.span_kind),
        custom,
    };

    // Get service name
    let service_name = match exporter.get_service_name() {
        Some(Value::String(name)) => Some(name.to_string()),
        _ => None,
    };

    // Build data section
    let data = InstanaSpanData {
        sdk,
        service: service_name,
    };

    Ok(data)
}

/// Build the from section of the InstanaSpan
fn build_from_section(exporter: &InstanaExporter) -> InstanaSpanFrom {
    let process_id = match exporter.get_process_pid() {
        Some(Value::I64(pid)) => Some(pid),
        _ => None,
    };

    let host_id = match exporter.get_host_id() {
        Some(Value::String(id)) => Some(id.to_string()),
        _ => None,
    };

    InstanaSpanFrom {
        process_id,
        host_id,
    }
}

/// Convert OpenTelemetry Value to serde_json::Value
fn convert_value_to_json(value: &Value) -> serde_json::Value {
    match value {
        Value::Bool(v) => json!(v),
        Value::I64(v) => json!(v),
        Value::F64(v) => json!(v),
        Value::String(v) => json!(v.as_str()),
        Value::Array(_) => {
            // Since we can't directly iterate over opentelemetry::Array,
            // convert it to a string representation for now
            json!(value.to_string())
        },
        _ => json!(null),
    }
}

/// Convert SpanKind to string representation
fn convert_span_kind_to_string(span_kind: &SpanKind) -> String {
    match span_kind {
        SpanKind::Producer | SpanKind::Server => "entry".to_string(),
        SpanKind::Internal => "intermediate".to_string(),
        SpanKind::Client | SpanKind::Consumer => "exit".to_string(),
    }
}

/// Serialize a batch of spans to JSON
pub fn serialize_batch(exporter: &InstanaExporter, batch: &[SpanData]) -> Result<bytes::Bytes> {
    let mut instana_spans = Vec::with_capacity(batch.len());

    for span in batch {
        let instana_span = convert_to_instana_span(exporter, span)?;
        instana_spans.push(instana_span);
    }

    let json_string = serde_json::to_string(&instana_spans)?;
    Ok(bytes::Bytes::from(json_string))
}

// Made with Bob
