use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Represents the data structure for an Instana span
#[derive(Debug, Serialize, Deserialize)]
pub struct InstanaSpan {
    // Core span fields
    #[serde(rename = "p", skip_serializing_if = "Option::is_none")]
    pub parent_span_id: Option<String>, // parent span ID (16 hex chars)
    #[serde(rename = "t")]
    pub trace_id: String, // trace ID (16 hex chars, right-most part)
    #[serde(rename = "s")]
    pub span_id: String, // span ID (16 hex chars)
    #[serde(rename = "n")]
    pub name: String, // name (always "sdk" for SDK spans)
    #[serde(rename = "k")]
    pub kind: i32, // kind (1=entry, 2=exit, 3=intermediate)
    #[serde(rename = "ts")]
    pub timestamp: u64, // timestamp in milliseconds
    #[serde(rename = "d")]
    pub duration: u64, // duration in milliseconds
    #[serde(rename = "sy")]
    pub synthetic: bool, // synthetic flag

    // Optional fields
    #[serde(rename = "lt", skip_serializing_if = "Option::is_none")]
    pub long_trace_id: Option<String>, // long trace ID (32 hex chars, only for entry spans)
    #[serde(rename = "ec", skip_serializing_if = "Option::is_none")]
    pub error_count: Option<i32>, // error count
    #[serde(rename = "crid", skip_serializing_if = "Option::is_none")]
    pub correlation_id: Option<String>, // correlation ID
    #[serde(rename = "crtp", skip_serializing_if = "Option::is_none")]
    pub correlation_type: Option<String>, // correlation type
    #[serde(rename = "tp", skip_serializing_if = "Option::is_none")]
    pub trace_parent: Option<bool>, // trace parent flag

    // Data section
    pub data: InstanaSpanData,

    // From section
    #[serde(rename = "f")]
    pub from: InstanaSpanFrom,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct InstanaSpanData {
    pub sdk: InstanaSdk,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub service: Option<String>, // Service name
}

#[derive(Debug, Serialize, Deserialize)]
pub struct InstanaSdk {
    pub name: String,
    #[serde(rename = "type")]
    pub span_type: String,
    pub custom: InstanaCustom,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct InstanaCustom {
    pub tags: InstanaTags,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct InstanaTags {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub attributes: Option<HashMap<String, serde_json::Value>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub resource: Option<HashMap<String, String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub events: Option<HashMap<String, InstanaEvent>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub links: Option<Vec<InstanaLink>>,
    pub otel: InstanaOtel,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct InstanaEvent {
    pub value: HashMap<String, serde_json::Value>,
    pub timestamp: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct InstanaLink {
    #[serde(rename = "t")]
    pub trace_id: String,
    #[serde(rename = "s")]
    pub span_id: String,
    pub attributes: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct InstanaOtel {
    #[serde(rename = "scope.name")]
    pub scope_name: String,
    #[serde(rename = "scope.version", skip_serializing_if = "Option::is_none")]
    pub scope_version: Option<String>,
    #[serde(rename = "dropped_events_count")]
    pub dropped_events_count: u32,
    #[serde(rename = "dropped_links_count")]
    pub dropped_links_count: u32,
    #[serde(rename = "dropped_attributes_count")]
    pub dropped_attributes_count: u32,
    #[serde(rename = "status_code", skip_serializing_if = "Option::is_none")]
    pub status_code: Option<String>,
    #[serde(rename = "status_description", skip_serializing_if = "Option::is_none")]
    pub status_description: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct InstanaSpanFrom {
    #[serde(rename = "e", skip_serializing_if = "Option::is_none")]
    pub process_id: Option<i64>, // process ID
    #[serde(rename = "h", skip_serializing_if = "Option::is_none")]
    pub host_id: Option<String>, // host ID
}

// Made with Bob
