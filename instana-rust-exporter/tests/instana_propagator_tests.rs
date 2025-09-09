use instana_opentelemetry_sdk::propagator::Propagator;
use opentelemetry::{
    propagation::{Extractor, Injector, TextMapPropagator},
    trace::{SpanContext, SpanId, TraceContextExt, TraceFlags, TraceId, TraceState},
    Context,
};
use std::collections::HashMap;

// Mock implementation of Extractor for testing
#[derive(Debug, Default)]
struct MockExtractor {
    data: HashMap<String, String>,
}

impl MockExtractor {
    fn new() -> Self {
        Self {
            data: HashMap::new(),
        }
    }

    fn with_header(mut self, key: &str, value: &str) -> Self {
        self.data.insert(key.to_string(), value.to_string());
        self
    }
}

impl Extractor for MockExtractor {
    fn get(&self, key: &str) -> Option<&str> {
        self.data.get(key).map(|s| s.as_str())
    }

    fn keys(&self) -> Vec<&str> {
        self.data.keys().map(|k| k.as_str()).collect()
    }
}

// Mock implementation of Injector for testing
#[derive(Debug, Default)]
struct MockInjector {
    data: HashMap<String, String>,
}

impl MockInjector {
    fn new() -> Self {
        Self {
            data: HashMap::new(),
        }
    }
}

impl Injector for MockInjector {
    fn set(&mut self, key: &str, value: String) {
        self.data.insert(key.to_string(), value);
    }
}

#[test]
fn test_fields() {
    let propagator = Propagator::new();
    let fields = propagator.fields().collect::<Vec<&str>>();

    assert_eq!(fields.len(), 3);
    assert!(fields.contains(&"X-INSTANA-T"));
    assert!(fields.contains(&"X-INSTANA-S"));
    assert!(fields.contains(&"X-INSTANA-L"));
}

#[test]
fn test_extract_valid_context() {
    let propagator = Propagator::new();
    let extractor = MockExtractor::new()
        .with_header("X-INSTANA-T", "1234567890abcdef1234567890abcdef")
        .with_header("X-INSTANA-S", "1234567890abcdef")
        .with_header("X-INSTANA-L", "1");

    let cx = Context::current();
    let extracted_cx = propagator.extract_with_context(&cx, &extractor);

    // Get the span context directly
    let span = extracted_cx.span();
    let sc = span.span_context();

    assert!(sc.is_valid());
    assert_eq!(
        sc.trace_id().to_string(),
        "1234567890abcdef1234567890abcdef"
    );
    assert_eq!(sc.span_id().to_string(), "1234567890abcdef");
    assert!(sc.is_remote());
    assert!(sc.is_sampled());
}

#[test]
fn test_extract_not_sampled() {
    let propagator = Propagator::new();
    let extractor = MockExtractor::new()
        .with_header("X-INSTANA-T", "1234567890abcdef1234567890abcdef")
        .with_header("X-INSTANA-S", "1234567890abcdef")
        .with_header("X-INSTANA-L", "0");

    let cx = Context::current();
    let extracted_cx = propagator.extract_with_context(&cx, &extractor);

    // Check directly
    let span = extracted_cx.span();
    let sc = span.span_context();
    assert!(sc.is_valid());
    assert!(!sc.is_sampled());
}

#[test]
fn test_extract_invalid_level() {
    let propagator = Propagator::new();
    let extractor = MockExtractor::new()
        .with_header("X-INSTANA-T", "1234567890abcdef1234567890abcdef")
        .with_header("X-INSTANA-S", "1234567890abcdef")
        .with_header("X-INSTANA-L", "invalid");

    let cx = Context::current();
    let extracted_cx = propagator.extract_with_context(&cx, &extractor);

    // Should default to not sampled
    let span = extracted_cx.span();
    let sc = span.span_context();
    assert!(sc.is_valid());
    assert!(!sc.is_sampled());
}

#[test]
fn test_extract_missing_trace_id() {
    let propagator = Propagator::new();
    let extractor = MockExtractor::new()
        .with_header("X-INSTANA-S", "1234567890abcdef")
        .with_header("X-INSTANA-L", "1");

    let cx = Context::current();
    let extracted_cx = propagator.extract_with_context(&cx, &extractor);

    // When extraction fails, the original context is returned
    // The original context has an invalid span context
    let span = extracted_cx.span();
    let sc = span.span_context();
    assert!(!sc.is_valid());
}

#[test]
fn test_extract_missing_span_id() {
    let propagator = Propagator::new();
    let extractor = MockExtractor::new()
        .with_header("X-INSTANA-T", "1234567890abcdef1234567890abcdef")
        .with_header("X-INSTANA-L", "1");

    let cx = Context::current();
    let extracted_cx = propagator.extract_with_context(&cx, &extractor);

    // When extraction fails, the original context is returned
    // The original context has an invalid span context
    let span = extracted_cx.span();
    let sc = span.span_context();
    assert!(!sc.is_valid());
}

#[test]
fn test_extract_missing_level() {
    let propagator = Propagator::new();
    let extractor = MockExtractor::new()
        .with_header("X-INSTANA-T", "1234567890abcdef1234567890abcdef")
        .with_header("X-INSTANA-S", "1234567890abcdef");

    let cx = Context::current();
    let extracted_cx = propagator.extract_with_context(&cx, &extractor);

    // When extraction fails, the original context is returned
    // The original context has an invalid span context
    let span = extracted_cx.span();
    let sc = span.span_context();
    assert!(!sc.is_valid());
}

#[test]
fn test_extract_invalid_trace_id_format() {
    let propagator = Propagator::new();
    let extractor = MockExtractor::new()
        .with_header("X-INSTANA-T", "invalid")
        .with_header("X-INSTANA-S", "1234567890abcdef")
        .with_header("X-INSTANA-L", "1");

    let cx = Context::current();
    let extracted_cx = propagator.extract_with_context(&cx, &extractor);

    // When extraction fails, the original context is returned
    // The original context has an invalid span context
    let span = extracted_cx.span();
    let sc = span.span_context();
    assert!(!sc.is_valid());
}

#[test]
fn test_extract_invalid_span_id_format() {
    let propagator = Propagator::new();
    let extractor = MockExtractor::new()
        .with_header("X-INSTANA-T", "1234567890abcdef1234567890abcdef")
        .with_header("X-INSTANA-S", "invalid")
        .with_header("X-INSTANA-L", "1");

    let cx = Context::current();
    let extracted_cx = propagator.extract_with_context(&cx, &extractor);

    // When extraction fails, the original context is returned
    // The original context has an invalid span context
    let span = extracted_cx.span();
    let sc = span.span_context();
    assert!(!sc.is_valid());
}

#[test]
fn test_extract_uppercase_trace_id() {
    let propagator = Propagator::new();
    let extractor = MockExtractor::new()
        .with_header("X-INSTANA-T", "1234567890ABCDEF1234567890ABCDEF")
        .with_header("X-INSTANA-S", "1234567890abcdef")
        .with_header("X-INSTANA-L", "1");

    let cx = Context::current();
    let extracted_cx = propagator.extract_with_context(&cx, &extractor);

    // When extraction fails, the original context is returned
    // The original context has an invalid span context
    let span = extracted_cx.span();
    let sc = span.span_context();
    assert!(!sc.is_valid());
}

#[test]
fn test_extract_uppercase_span_id() {
    let propagator = Propagator::new();
    let extractor = MockExtractor::new()
        .with_header("X-INSTANA-T", "1234567890abcdef1234567890abcdef")
        .with_header("X-INSTANA-S", "1234567890ABCDEF")
        .with_header("X-INSTANA-L", "1");

    let cx = Context::current();
    let extracted_cx = propagator.extract_with_context(&cx, &extractor);

    // When extraction fails, the original context is returned
    // The original context has an invalid span context
    let span = extracted_cx.span();
    let sc = span.span_context();
    assert!(!sc.is_valid());
}

#[test]
fn test_extract_zero_trace_id() {
    let propagator = Propagator::new();
    let extractor = MockExtractor::new()
        .with_header("X-INSTANA-T", "00000000000000000000000000000000")
        .with_header("X-INSTANA-S", "1234567890abcdef")
        .with_header("X-INSTANA-L", "1");

    let cx = Context::current();
    let extracted_cx = propagator.extract_with_context(&cx, &extractor);

    // When extraction fails, the original context is returned
    // The original context has an invalid span context
    let span = extracted_cx.span();
    let sc = span.span_context();
    assert!(!sc.is_valid());
}

#[test]
fn test_extract_zero_span_id() {
    let propagator = Propagator::new();
    let extractor = MockExtractor::new()
        .with_header("X-INSTANA-T", "1234567890abcdef1234567890abcdef")
        .with_header("X-INSTANA-S", "0000000000000000")
        .with_header("X-INSTANA-L", "1");

    let cx = Context::current();
    let extracted_cx = propagator.extract_with_context(&cx, &extractor);

    // When extraction fails, the original context is returned
    // The original context has an invalid span context
    let span = extracted_cx.span();
    let sc = span.span_context();
    assert!(!sc.is_valid());
}

#[test]
fn test_inject_valid_context() {
    let propagator = Propagator::new();
    let mut injector = MockInjector::new();

    let trace_id = TraceId::from_hex("1234567890abcdef1234567890abcdef").unwrap();
    let span_id = SpanId::from_hex("1234567890abcdef").unwrap();
    let span_context = SpanContext::new(
        trace_id,
        span_id,
        TraceFlags::SAMPLED,
        false,
        TraceState::default(),
    );

    // Create a context with the span context
    let cx = Context::current().with_remote_span_context(span_context);
    propagator.inject_context(&cx, &mut injector);

    assert_eq!(
        injector.data.get("X-INSTANA-T").unwrap(),
        "1234567890abcdef1234567890abcdef"
    );
    assert_eq!(
        injector.data.get("X-INSTANA-S").unwrap(),
        "1234567890abcdef"
    );
    assert_eq!(injector.data.get("X-INSTANA-L").unwrap(), "1");
}

#[test]
fn test_inject_not_sampled_context() {
    let propagator = Propagator::new();
    let mut injector = MockInjector::new();

    let trace_id = TraceId::from_hex("1234567890abcdef1234567890abcdef").unwrap();
    let span_id = SpanId::from_hex("1234567890abcdef").unwrap();
    let span_context = SpanContext::new(
        trace_id,
        span_id,
        TraceFlags::default(), // NOT_SAMPLED
        false,
        TraceState::default(),
    );

    // Create a context with the span context
    let cx = Context::current().with_remote_span_context(span_context);
    propagator.inject_context(&cx, &mut injector);

    assert_eq!(
        injector.data.get("X-INSTANA-T").unwrap(),
        "1234567890abcdef1234567890abcdef"
    );
    assert_eq!(
        injector.data.get("X-INSTANA-S").unwrap(),
        "1234567890abcdef"
    );
    assert_eq!(injector.data.get("X-INSTANA-L").unwrap(), "0");
}

#[test]
fn test_inject_invalid_context() {
    let propagator = Propagator::new();
    let mut injector = MockInjector::new();

    // Use an invalid context (default)
    let cx = Context::current();
    propagator.inject_context(&cx, &mut injector);

    // Should not inject anything for invalid context
    assert!(injector.data.is_empty());
}

#[test]
fn test_roundtrip() {
    let propagator = Propagator::new();

    // Create a valid context
    let trace_id = TraceId::from_hex("1234567890abcdef1234567890abcdef").unwrap();
    let span_id = SpanId::from_hex("1234567890abcdef").unwrap();
    let span_context = SpanContext::new(
        trace_id,
        span_id,
        TraceFlags::SAMPLED,
        false,
        TraceState::default(),
    );

    // Create a context with the span context
    let original_cx = Context::current().with_remote_span_context(span_context);

    // Inject it
    let mut injector = MockInjector::new();
    propagator.inject_context(&original_cx, &mut injector);

    // Create an extractor with the same data
    let extractor = MockExtractor::new()
        .with_header("X-INSTANA-T", injector.data.get("X-INSTANA-T").unwrap())
        .with_header("X-INSTANA-S", injector.data.get("X-INSTANA-S").unwrap())
        .with_header("X-INSTANA-L", injector.data.get("X-INSTANA-L").unwrap());

    // Extract it back
    let empty_cx = Context::current();
    let extracted_cx = propagator.extract_with_context(&empty_cx, &extractor);

    // Compare the span contexts
    let span = extracted_cx.span();
    let extracted_sc = span.span_context();

    assert_eq!(extracted_sc.trace_id(), trace_id);
    assert_eq!(extracted_sc.span_id(), span_id);
    assert_eq!(extracted_sc.trace_flags(), TraceFlags::SAMPLED);
    assert!(extracted_sc.is_remote()); // Note: This will be true in extracted context
}
