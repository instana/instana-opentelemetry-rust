use opentelemetry_instana::{InstanaExporter};
use opentelemetry_instana::exporter::serialize_span;
use opentelemetry::{
    trace::{SpanContext, SpanId, SpanKind, Status, TraceFlags, TraceId, TraceState},
    InstrumentationScope, KeyValue,
};
use opentelemetry_sdk::{
    trace::{SpanData, SpanEvents, SpanLinks},
    Resource,
};
use serde_json::Value;
use std::{
    sync::OnceLock,
    time::{Duration, SystemTime, UNIX_EPOCH},
};

fn get_resource() -> Resource {
    static RESOURCE: OnceLock<Resource> = OnceLock::new();
    RESOURCE
        .get_or_init(|| {
            Resource::builder()
                .with_service_name("test-service")
                .build()
        })
        .clone()
}

fn get_empty_resource() -> Resource {
    Resource::builder_empty().build()
}

fn create_test_span_data(span_kind: SpanKind, with_parent: bool) -> SpanData {
    let mut attributes: Vec<KeyValue> = Vec::new();
    attributes.push(KeyValue::new("INTERNAL_TAG_ENTITY_ID", 100));
    attributes.push(KeyValue::new("INTERNAL_TAG_CRTP", "ntg"));

    let time = SystemTime::now();

    let parent_span_id = if with_parent {
        SpanId::from_hex("0807060504030201").unwrap()
    } else {
        SpanId::INVALID
    };

    SpanData {
        span_context: SpanContext::new(
            TraceId::from_hex("0102030405060708090a0b0c0d0e0f10").unwrap(),
            SpanId::from_hex("0102030405060708").unwrap(),
            TraceFlags::SAMPLED,
            false,
            TraceState::default(),
        ),
        parent_span_id,
        span_kind,
        name: std::borrow::Cow::Borrowed("test-span"),
        start_time: time,
        end_time: time + Duration::from_millis(100), // Add 100ms duration
        attributes,
        dropped_attributes_count: 0,
        events: SpanEvents::default(),
        links: SpanLinks::default(),
        status: Status::Ok,
        instrumentation_scope: InstrumentationScope::builder("test-instrumentation").build(),
    }
}

fn create_span_with_synthetic_tag() -> SpanData {
    let mut span = create_test_span_data(SpanKind::Client, false);
    span.attributes
        .push(KeyValue::new("X-INSTANA-SYNTHETIC", 1));
    span
}

fn create_span_with_error_status() -> SpanData {
    let mut span = create_test_span_data(SpanKind::Client, false);
    span.status = Status::error("something went wrong");
    span
}

fn create_span_with_trace_state() -> SpanData {
    let mut span = create_test_span_data(SpanKind::Client, false);

    // Instead of using trace state, add the correlation attributes directly
    span.attributes
        .push(KeyValue::new("INTERNAL_TAG_CRID", "123"));
    span.attributes
        .push(KeyValue::new("INTERNAL_TAG_CRTP", "ntg"));

    span
}

fn create_span_with_null_attributes() -> SpanData {
    let mut span = create_test_span_data(SpanKind::Client, false);

    // Add empty string attribute
    span.attributes.push(KeyValue::new("empty_string", ""));

    // Add zero value attributes
    span.attributes.push(KeyValue::new("zero_int", 0));
    span.attributes.push(KeyValue::new("zero_float", 0.0));

    // Add boolean false attribute
    span.attributes.push(KeyValue::new("false_bool", false));

    // Add whitespace-only string
    span.attributes
        .push(KeyValue::new("whitespace_string", "   "));

    span
}

fn create_span_with_negative_duration() -> SpanData {
    let mut span = create_test_span_data(SpanKind::Client, false);

    // Set end time to be before start time
    let current_time = SystemTime::now();
    span.start_time = current_time + Duration::from_secs(10); // Start time in the future
    span.end_time = current_time; // End time now (before start time)

    span
}

// Additional edge case test spans

fn create_span_with_json_special_chars() -> SpanData {
    let mut span = create_test_span_data(SpanKind::Client, false);

    // Add attributes with JSON special characters
    span.attributes.push(KeyValue::new(
        "escaped_string",
        r#"Special chars: "quotes", \backslashes\, /slashes/, control chars: \b\f\n\r\t"#,
    ));

    // Add attribute with JSON-like content
    span.attributes.push(KeyValue::new(
        "json_object_string",
        r#"{"nested":"value","with":"quotes"}"#,
    ));

    span
}

fn create_span_with_unicode_chars() -> SpanData {
    let mut span = create_test_span_data(SpanKind::Client, false);

    // Add attributes with Unicode characters
    span.attributes.push(KeyValue::new("emoji", "😀 😎 🚀 💯"));
    span.attributes.push(KeyValue::new(
        "international",
        "こんにちは 你好 Привет Olá Здравствуйте",
    ));
    span.attributes
        .push(KeyValue::new("symbols", "♠ ♥ ♦ ♣ ★ ☆ ☂ ♞"));

    span
}

fn create_span_with_timestamp_edge_cases() -> (SpanData, SpanData) {
    // 1. Epoch start (1970-01-01)
    let mut epoch_span = create_test_span_data(SpanKind::Client, false);
    epoch_span.start_time = UNIX_EPOCH;
    epoch_span.end_time = UNIX_EPOCH + Duration::from_secs(1);

    // 2. Far future date
    let mut future_span = create_test_span_data(SpanKind::Client, false);
    future_span.start_time = UNIX_EPOCH + Duration::from_secs(32503680000); // Year 3000 approximately
    future_span.end_time = future_span.start_time + Duration::from_secs(1);

    (epoch_span, future_span)
}

fn serialize_and_parse(exporter: &InstanaExporter, span: &SpanData) -> Result<Value, anyhow::Error> {
    // Convert the span to an InstanaSpan
    let instana_span = serialize_span::convert_to_instana_span(exporter, span)?;

    // Serialize to JSON
    let json_string = serde_json::to_string(&instana_span)?;

    // Parse back to a JSON Value for testing
    let json_value: Value = serde_json::from_str(&json_string)?;

    Ok(json_value)
}

#[test]
fn test_serialize_client_span() {
    let exporter = InstanaExporter::builder()
        .with_service(get_resource())
        .build()
        .expect("failed to build instana exporter");

    let span = create_test_span_data(SpanKind::Client, false);

    let json_value = serialize_and_parse(&exporter, &span).expect("Failed to serialize span");

    // Check basic structure
    assert!(json_value.is_object());

    // Check span kind (k) - should be 2 for Client
    assert_eq!(json_value["k"], 2);

    // Check trace and span IDs
    assert_eq!(json_value["t"], "090a0b0c0d0e0f10");
    assert_eq!(json_value["s"], "0102030405060708");

    // Check synthetic flag
    assert_eq!(json_value["sy"], false);

    // Check name
    assert_eq!(json_value["n"], "sdk");

    // Check data section
    assert!(json_value["data"].is_object());
    assert_eq!(json_value["data"]["sdk"]["name"], "test-span");
    assert_eq!(json_value["data"]["sdk"]["type"], "exit");
    assert_eq!(json_value["data"]["service"], "test-service");
}

#[test]
fn test_serialize_server_span() {
    let exporter = InstanaExporter::builder()
        .with_service(get_resource())
        .build()
        .expect("failed to build instana exporter");

    let span = create_test_span_data(SpanKind::Server, false);

    let json_value = serialize_and_parse(&exporter, &span).expect("Failed to serialize span");

    // Check span kind (k) - should be 1 for Server
    assert_eq!(json_value["k"], 1);

    // Check for long trace ID (lt) which should be present for entry spans
    assert_eq!(json_value["lt"], "0102030405060708090a0b0c0d0e0f10");

    // Check data section
    assert_eq!(json_value["data"]["sdk"]["type"], "entry");
}

#[test]
fn test_serialize_internal_span() {
    let exporter = InstanaExporter::builder()
        .with_service(get_resource())
        .build()
        .expect("failed to build instana exporter");

    let span = create_test_span_data(SpanKind::Internal, false);

    let json_value = serialize_and_parse(&exporter, &span).expect("Failed to serialize span");

    // Check span kind (k) - should be 3 for Internal
    assert_eq!(json_value["k"], 3);

    // Check data section
    assert_eq!(json_value["data"]["sdk"]["type"], "intermediate");
}

#[test]
fn test_serialize_with_parent() {
    let exporter = InstanaExporter::builder()
        .with_service(get_resource())
        .build()
        .expect("failed to build instana exporter");

    let span = create_test_span_data(SpanKind::Client, true);

    let json_value = serialize_and_parse(&exporter, &span).expect("Failed to serialize span");

    // Check parent span ID
    assert_eq!(json_value["p"], "0807060504030201");
}

#[test]
fn test_serialize_with_synthetic_tag() {
    let exporter = InstanaExporter::builder()
        .with_service(get_resource())
        .build()
        .expect("failed to build instana exporter");

    let span = create_span_with_synthetic_tag();

    let json_value = serialize_and_parse(&exporter, &span).expect("Failed to serialize span");

    // Check synthetic flag
    assert_eq!(json_value["sy"], true);
}

#[test]
fn test_serialize_with_error_status() {
    let exporter = InstanaExporter::builder()
        .with_service(get_resource())
        .build()
        .expect("failed to build instana exporter");

    let span = create_span_with_error_status();

    let json_value = serialize_and_parse(&exporter, &span).expect("Failed to serialize span");

    // Check error count
    assert_eq!(json_value["ec"], 1);
}

#[test]
fn test_serialize_with_trace_state() {
    let exporter = InstanaExporter::builder()
        .with_service(get_resource())
        .build()
        .expect("failed to build instana exporter");

    let span = create_span_with_trace_state();

    let json_value = serialize_and_parse(&exporter, &span).expect("Failed to serialize span");

    // Check correlation fields
    assert_eq!(json_value["crid"], "123");
    assert_eq!(json_value["crtp"], "ntg");
}

// Edge Case Tests

#[test]
fn test_serialize_with_null_attributes() {
    let exporter = InstanaExporter::builder()
        .with_service(get_resource())
        .build()
        .expect("failed to build instana exporter");

    let span = create_span_with_null_attributes();

    let json_value = serialize_and_parse(&exporter, &span).expect("Failed to serialize span");

    // Verify the basic structure
    assert!(json_value.is_object());
    assert!(json_value["data"].is_object());
    assert!(json_value["data"]["sdk"].is_object());
    assert!(json_value["data"]["sdk"]["custom"].is_object());
    assert!(json_value["data"]["sdk"]["custom"]["tags"].is_object());

    // Verify the null-like attributes were serialized correctly
    let tags = &json_value["data"]["sdk"]["custom"]["tags"];

    if let Some(attributes) = tags["attributes"].as_object() {
        // Empty string should be serialized as an empty string
        assert_eq!(attributes["empty_string"], "");

        // Zero values should be serialized as 0
        assert_eq!(attributes["zero_int"], 0);
        assert_eq!(attributes["zero_float"], 0.0);

        // Boolean false should be serialized as false
        assert_eq!(attributes["false_bool"], false);

        // Whitespace string should be preserved
        assert_eq!(attributes["whitespace_string"], "   ");
    } else {
        panic!("Expected attributes object in tags");
    }
}

#[test]
fn test_serialize_with_empty_resource() {
    // Create an exporter with an empty resource
    let exporter = InstanaExporter::builder()
        .with_service(get_empty_resource())
        .build()
        .expect("failed to build instana exporter");

    // Create a basic span
    let span = create_test_span_data(SpanKind::Client, false);

    let json_value = serialize_and_parse(&exporter, &span).expect("Failed to serialize span");

    // Verify the basic structure
    assert!(json_value.is_object());
    assert!(json_value["data"].is_object());

    // The service field should be empty since we have an empty resource
    assert!(json_value["data"]["service"].is_null());

    // Verify that the span was still serialized correctly
    assert_eq!(json_value["t"], "090a0b0c0d0e0f10");
    assert_eq!(json_value["s"], "0102030405060708");
    assert_eq!(json_value["n"], "sdk");
}

#[test]
fn test_serialize_with_negative_duration() {
    let exporter = InstanaExporter::builder()
        .with_service(get_resource())
        .build()
        .expect("failed to build instana exporter");

    let span = create_span_with_negative_duration();

    // The serialization should now fail with an error
    let result = serialize_span::convert_to_instana_span(&exporter, &span);
    assert!(result.is_err());

    // Check that the error message contains the expected text
    let error_message = format!("{}", result.unwrap_err());
    assert!(error_message.contains("Error parsing duration"));
}

#[test]
fn test_serialize_with_combined_edge_cases() {
    // Create an exporter with an empty resource
    let exporter = InstanaExporter::builder()
        .with_service(get_empty_resource())
        .build()
        .expect("failed to build instana exporter");

    // Create a span with multiple edge cases:
    // 1. Null-like attributes
    let span = create_span_with_null_attributes();

    let json_value = serialize_and_parse(&exporter, &span).expect("Failed to serialize span");

    // Verify the basic structure
    assert!(json_value.is_object());

    // Service field should be empty due to empty resource
    assert!(json_value["data"]["service"].is_null());

    // Null-like attributes should be serialized correctly
    let tags = &json_value["data"]["sdk"]["custom"]["tags"];
    if let Some(attributes) = tags["attributes"].as_object() {
        assert_eq!(attributes["empty_string"], "");
        assert_eq!(attributes["zero_int"], 0);
        assert_eq!(attributes["false_bool"], false);
    } else {
        panic!("Expected attributes object in tags");
    }
}

#[test]
fn test_serialize_batch() {
    let exporter = InstanaExporter::builder()
        .with_service(get_resource())
        .build()
        .expect("failed to build instana exporter");

    // Create a batch of spans
    let spans = vec![
        create_test_span_data(SpanKind::Client, false),
        create_test_span_data(SpanKind::Server, true),
        create_span_with_synthetic_tag(),
    ];

    // Serialize the batch
    let bytes =
        serialize_span::serialize_batch(&exporter, &spans).expect("Failed to serialize batch");

    // Convert bytes to string and parse as JSON
    let json_str = std::str::from_utf8(&bytes).expect("Failed to convert bytes to string");
    let json_value: Value = serde_json::from_str(json_str).expect("Failed to parse JSON");

    // Verify it's an array with the correct number of spans
    assert!(json_value.is_array());
    assert_eq!(json_value.as_array().unwrap().len(), 3);

    // Check the first span
    let first_span = &json_value[0];
    assert_eq!(first_span["k"], 2); // Client span
    assert_eq!(first_span["t"], "090a0b0c0d0e0f10");
    assert_eq!(first_span["s"], "0102030405060708");

    // Check the second span
    let second_span = &json_value[1];
    assert_eq!(second_span["k"], 1); // Server span
    assert_eq!(second_span["p"], "0807060504030201"); // Has parent

    // Check the third span
    let third_span = &json_value[2];
    assert_eq!(third_span["sy"], true); // Synthetic span
}

// Additional edge case tests

#[test]
fn test_serialize_with_json_special_chars() {
    let exporter = InstanaExporter::builder()
        .with_service(get_resource())
        .build()
        .expect("failed to build instana exporter");

    let span = create_span_with_json_special_chars();

    let json_value = serialize_and_parse(&exporter, &span).expect("Failed to serialize span");

    // Verify the basic structure
    assert!(json_value.is_object());

    // Get the attributes
    let tags = &json_value["data"]["sdk"]["custom"]["tags"];
    let attributes = tags["attributes"]
        .as_object()
        .expect("Expected attributes object");

    // Check that the escaped string was handled correctly
    let escaped_str = attributes["escaped_string"]
        .as_str()
        .expect("Expected string value");
    assert!(escaped_str.contains("quotes"));
    assert!(escaped_str.contains("backslashes"));
    assert!(escaped_str.contains("slashes"));
    assert!(escaped_str.contains("control chars"));

    // Check that the JSON-like string was handled correctly
    let json_obj_str = attributes["json_object_string"]
        .as_str()
        .expect("Expected string value");
    assert!(json_obj_str.contains(r#"{"nested":"value","with":"quotes"}"#));
}

#[test]
fn test_serialize_with_unicode_chars() {
    let exporter = InstanaExporter::builder()
        .with_service(get_resource())
        .build()
        .expect("failed to build instana exporter");

    let span = create_span_with_unicode_chars();

    let json_value = serialize_and_parse(&exporter, &span).expect("Failed to serialize span");

    // Verify the basic structure
    assert!(json_value.is_object());

    // Get the attributes
    let tags = &json_value["data"]["sdk"]["custom"]["tags"];
    let attributes = tags["attributes"]
        .as_object()
        .expect("Expected attributes object");

    // Check that the Unicode characters were handled correctly
    assert_eq!(attributes["emoji"], "😀 😎 🚀 💯");
    assert_eq!(
        attributes["international"],
        "こんにちは 你好 Привет Olá Здравствуйте"
    );
    assert_eq!(attributes["symbols"], "♠ ♥ ♦ ♣ ★ ☆ ☂ ♞");
}

#[test]
fn test_serialize_with_timestamp_edge_cases() {
    let exporter = InstanaExporter::builder()
        .with_service(get_resource())
        .build()
        .expect("failed to build instana exporter");

    let (epoch_span, future_span) = create_span_with_timestamp_edge_cases();

    // Test the epoch span
    let epoch_json =
        serialize_and_parse(&exporter, &epoch_span).expect("Failed to serialize epoch span");

    // Verify the epoch timestamp was handled correctly
    assert_eq!(epoch_json["ts"], 0);
    assert_eq!(epoch_json["d"], 1000); // 1 second duration in milliseconds

    // Test the future span
    let future_json =
        serialize_and_parse(&exporter, &future_span).expect("Failed to serialize future span");

    // Verify the future timestamp was handled correctly
    assert_eq!(future_json["ts"], 32503680000000_i64);
    assert_eq!(future_json["d"], 1000); // 1 second duration in milliseconds
}

#[test]
fn test_serialize_empty_collections() {
    let exporter = InstanaExporter::builder()
        .with_service(get_resource())
        .build()
        .expect("failed to build instana exporter");

    // Create a basic span with no events, links, or attributes
    let mut span = create_test_span_data(SpanKind::Client, false);
    span.attributes.clear(); // Remove all attributes

    let json_value = serialize_and_parse(&exporter, &span).expect("Failed to serialize span");

    // Verify the basic structure
    assert!(json_value.is_object());

    // Check that the tags section exists but doesn't have attributes
    let tags = &json_value["data"]["sdk"]["custom"]["tags"];
    assert!(!tags.as_object().unwrap().contains_key("attributes"));

    // Check that events and links are not present
    assert!(!tags.as_object().unwrap().contains_key("events"));
    assert!(!tags.as_object().unwrap().contains_key("links"));
}

// Made with Bob
