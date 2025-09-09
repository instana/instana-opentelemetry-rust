use opentelemetry::{
    propagation::{text_map_propagator::FieldIter, Extractor, Injector, TextMapPropagator},
    trace::{SpanContext, SpanId, TraceContextExt, TraceFlags, TraceId, TraceState},
    Context,
};

use anyhow::Context as anyhow_context;
use anyhow::{bail, Result};
use std::sync::OnceLock;

const INSTANA_TRACE_ID_HEADER: &str = "X-INSTANA-T";
const INSTANA_SPAN_ID_HEADER: &str = "X-INSTANA-S";
const INSTANA_LEVEL_HEADER: &str = "X-INSTANA-L";
const IS_SAMPLED: &str = "1";
const IS_NOT_SAMPLED: &str = "0";

static INSTANA_CONTEXT_HEADER_FIELDS: OnceLock<[String; 3]> = OnceLock::new();

fn instana_context_header_fields() -> &'static [String; 3] {
    INSTANA_CONTEXT_HEADER_FIELDS.get_or_init(|| {
        [
            INSTANA_TRACE_ID_HEADER.to_owned(),
            INSTANA_SPAN_ID_HEADER.to_owned(),
            INSTANA_LEVEL_HEADER.to_owned(),
        ]
    })
}

#[derive(Debug)]
pub struct Propagator {
    _private: (),
}

impl Propagator {
    pub fn new() -> Self {
        Propagator { _private: () }
    }

    fn extract_span_context(&self, extractor: &dyn Extractor) -> Result<SpanContext> {
        // Extract and validate trace ID
        let trace_id_str = extractor.get(INSTANA_TRACE_ID_HEADER).ok_or_else(|| {
            anyhow::anyhow!("Missing trace ID header: {}", INSTANA_TRACE_ID_HEADER)
        })?;

        // Fast path check for uppercase characters
        if trace_id_str.chars().any(|c| c.is_ascii_uppercase()) {
            bail!("TraceId must be lowercase: {}", trace_id_str);
        }

        let trace_id = TraceId::from_hex(trace_id_str)
            .context(format!("Invalid Trace Id: {}", trace_id_str))?;

        // Extract and validate span ID
        let span_id_str = extractor
            .get(INSTANA_SPAN_ID_HEADER)
            .ok_or_else(|| anyhow::anyhow!("Missing span ID header: {}", INSTANA_SPAN_ID_HEADER))?;

        // Fast path check for uppercase characters
        if span_id_str.chars().any(|c| c.is_ascii_uppercase()) {
            bail!("SpanId must be lowercase: {}", span_id_str);
        }

        let span_id =
            SpanId::from_hex(span_id_str).context(format!("Invalid span id: {}", span_id_str))?;

        // Extract and determine sampling flag
        let level_value = extractor
            .get(INSTANA_LEVEL_HEADER)
            .ok_or_else(|| anyhow::anyhow!("Missing level header: {}", INSTANA_LEVEL_HEADER))?;

        let trace_flags = match level_value {
            IS_SAMPLED => TraceFlags::SAMPLED,
            _ => TraceFlags::NOT_SAMPLED,
        };

        // Create and validate span context
        let span_context = SpanContext::new(
            trace_id,
            span_id,
            trace_flags,
            true, // This is a remote context
            TraceState::NONE,
        );

        if !span_context.is_valid() {
            bail!(
                "Invalid span context: trace_id={}, span_id={}",
                trace_id,
                span_id
            );
        }

        Ok(span_context)
    }
}

impl TextMapPropagator for Propagator {
    /// convert the trace-id,span-id and sampling flag from span context into key-value pairs

    fn inject_context(&self, cx: &Context, injector: &mut dyn Injector) {
        let span = cx.span();
        let span_context = span.span_context();
        if span_context.is_valid() {
            injector.set(INSTANA_TRACE_ID_HEADER, span_context.trace_id().to_string());
            injector.set(INSTANA_SPAN_ID_HEADER, span_context.span_id().to_string());
            let level_value = if span_context.is_sampled() {
                IS_SAMPLED
            } else {
                IS_NOT_SAMPLED
            };
            injector.set(INSTANA_LEVEL_HEADER, level_value.to_string());
        }
    }

    fn extract_with_context(&self, cx: &Context, extractor: &dyn Extractor) -> Context {
        self.extract_span_context(extractor)
            .map(|sc| cx.with_remote_span_context(sc))
            .unwrap_or_else(|_| cx.clone())
    }

    fn fields(&self) -> FieldIter<'_> {
        FieldIter::new(instana_context_header_fields())
    }
}
