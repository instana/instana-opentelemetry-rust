use opentelemetry_instana::exporter::span_data::GET;
use opentelemetry::{
    trace::{Event, Link, SpanContext, SpanId, SpanKind, Status, TraceFlags, TraceId, TraceState},
    InstrumentationScope, KeyValue, Value,
};
use opentelemetry_sdk::trace::{SpanData, SpanEvents, SpanLinks};
use std::time::SystemTime;

fn create_test_span_data() -> SpanData {
    let mut attributes: Vec<KeyValue> = Vec::new();
    attributes.push(KeyValue::new("test_key1", "test_value1"));
    attributes.push(KeyValue::new("test_key2", 123));
    attributes.push(KeyValue::new("test_key3", true));

    let time = SystemTime::now();

    // Create events
    let event1 = Event::new(
        "event1".to_string(),
        time,
        vec![KeyValue::new("event_attr1", "event_value1")],
        0,
    );
    let event2 = Event::new(
        "event2".to_string(),
        time,
        vec![KeyValue::new("event_attr2", "event_value2")],
        0,
    );
    let events = vec![event1, event2];

    // Create links
    let link_context = SpanContext::new(
        TraceId::from_hex("0102030405060708090a0b0c0d0e0f10").unwrap(),
        SpanId::from_hex("0102030405060708").unwrap(),
        TraceFlags::SAMPLED,
        false,
        TraceState::default(),
    );
    let link1 = Link::new(
        link_context,
        vec![KeyValue::new("link_attr1", "link_value1")],
        0,
    );
    let links = vec![link1];

    let mut span_events = SpanEvents::default();
    span_events.events = events;

    let mut span_links = SpanLinks::default();
    span_links.links = links;

    SpanData {
        span_context: SpanContext::new(
            TraceId::from_hex("1102030405060708090a0b0c0d0e0f11").unwrap(),
            SpanId::from_hex("1102030405060708").unwrap(),
            TraceFlags::SAMPLED,
            false,
            TraceState::default(),
        ),
        parent_span_id: SpanId::from_hex("1807060504030201").unwrap(),
        span_kind: SpanKind::Client,
        name: std::borrow::Cow::Borrowed("test-span"),
        start_time: time,
        end_time: time,
        attributes,
        dropped_attributes_count: 0,
        events: span_events,
        links: span_links,
        status: Status::Ok,
        instrumentation_scope: InstrumentationScope::builder("test-instrumentation").build(),
    }
}

fn create_span_with_special_char_keys() -> SpanData {
    let mut span_data = create_test_span_data();

    // Clear existing attributes and add ones with special characters
    span_data.attributes.clear();
    span_data
        .attributes
        .push(KeyValue::new("key-with-dashes", "value1"));
    span_data
        .attributes
        .push(KeyValue::new("key.with.dots", "value2"));
    span_data
        .attributes
        .push(KeyValue::new("key with spaces", "value3"));
    span_data
        .attributes
        .push(KeyValue::new("key_with_underscores", "value4"));
    span_data
        .attributes
        .push(KeyValue::new("key@with@at@symbols", "value5"));
    span_data
        .attributes
        .push(KeyValue::new("key/with/slashes", "value6"));
    span_data
        .attributes
        .push(KeyValue::new("key:with:colons", "value7"));

    span_data
}

fn create_span_with_duplicate_keys() -> SpanData {
    let mut span_data = create_test_span_data();

    // Clear existing attributes and add duplicates
    span_data.attributes.clear();
    span_data
        .attributes
        .push(KeyValue::new("duplicate_key", "first_value"));
    span_data
        .attributes
        .push(KeyValue::new("unique_key", "unique_value"));
    span_data
        .attributes
        .push(KeyValue::new("duplicate_key", "second_value"));
    span_data
        .attributes
        .push(KeyValue::new("another_key", "another_value"));
    span_data
        .attributes
        .push(KeyValue::new("duplicate_key", "third_value"));

    span_data
}

fn create_span_with_empty_values() -> SpanData {
    let mut span_data = create_test_span_data();

    // Clear existing attributes and add ones with empty values
    span_data.attributes.clear();
    span_data.attributes.push(KeyValue::new("empty_string", ""));
    span_data.attributes.push(KeyValue::new("zero_int", 0));
    span_data.attributes.push(KeyValue::new("zero_float", 0.0));
    span_data
        .attributes
        .push(KeyValue::new("false_bool", false));
    span_data
        .attributes
        .push(KeyValue::new("whitespace_string", "   "));

    // Add a normal attribute for comparison
    span_data
        .attributes
        .push(KeyValue::new("normal_key", "normal_value"));

    span_data
}

#[test]
fn test_get_attribute_existing() {
    let span_data = create_test_span_data();

    let result = span_data.get_attribute("test_key1").unwrap();
    assert_eq!(result, Value::String("test_value1".into()));

    let result = span_data.get_attribute("test_key2").unwrap();
    assert_eq!(result, Value::I64(123));

    let result = span_data.get_attribute("test_key3").unwrap();
    assert_eq!(result, Value::Bool(true));
}

#[test]
fn test_get_attribute_non_existing() {
    let span_data = create_test_span_data();

    let result = span_data.get_attribute("non_existing_key");
    assert!(result.is_err());
}

#[test]
fn test_get_attributes() {
    let span_data = create_test_span_data();

    let attributes = span_data.get_attributes();
    assert_eq!(attributes.len(), 3);

    // Check if all original attributes are present
    assert!(attributes.iter().any(
        |kv| kv.key.as_str() == "test_key1" && kv.value == Value::String("test_value1".into())
    ));
    assert!(attributes
        .iter()
        .any(|kv| kv.key.as_str() == "test_key2" && kv.value == Value::I64(123)));
    assert!(attributes
        .iter()
        .any(|kv| kv.key.as_str() == "test_key3" && kv.value == Value::Bool(true)));
}

#[test]
fn test_get_events() {
    let span_data = create_test_span_data();

    let events = span_data.get_events();
    assert_eq!(events.len(), 2);

    // Check if events have correct names
    assert!(events.iter().any(|e| e.name == "event1"));
    assert!(events.iter().any(|e| e.name == "event2"));

    // Check if event attributes are preserved
    let event1 = events.iter().find(|e| e.name == "event1").unwrap();
    assert_eq!(event1.attributes.len(), 1);
    assert_eq!(event1.attributes[0].key.as_str(), "event_attr1");
    assert_eq!(
        event1.attributes[0].value,
        Value::String("event_value1".into())
    );
}

#[test]
fn test_get_links() {
    let span_data = create_test_span_data();

    let links = span_data.get_links();
    assert_eq!(links.len(), 1);

    // Check if link attributes are preserved
    let link = &links[0];
    assert_eq!(link.attributes.len(), 1);
    assert_eq!(link.attributes[0].key.as_str(), "link_attr1");
    assert_eq!(
        link.attributes[0].value,
        Value::String("link_value1".into())
    );

    // Check if link context is preserved
    let trace_id = link.span_context.trace_id();
    let span_id = link.span_context.span_id();

    // Format the IDs as hex strings for comparison
    assert_eq!(
        format!("{:032x}", trace_id),
        "0102030405060708090a0b0c0d0e0f10"
    );
    assert_eq!(format!("{:016x}", span_id), "0102030405060708");
}

#[test]
fn test_get_attribute_case_sensitivity() {
    let span_data = create_test_span_data();

    // Original key is "test_key1" (lowercase)
    // Try to access with different case
    let result = span_data.get_attribute("TEST_KEY1");

    // The implementation should be case-sensitive, so this should fail
    assert!(result.is_err());

    // Verify the original key still works
    let result = span_data.get_attribute("test_key1");
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), Value::String("test_value1".into()));
}

#[test]
fn test_get_attribute_with_special_characters() {
    let span_data = create_span_with_special_char_keys();

    // Test retrieving attributes with special characters in keys
    let result = span_data.get_attribute("key-with-dashes").unwrap();
    assert_eq!(result, Value::String("value1".into()));

    let result = span_data.get_attribute("key.with.dots").unwrap();
    assert_eq!(result, Value::String("value2".into()));

    let result = span_data.get_attribute("key with spaces").unwrap();
    assert_eq!(result, Value::String("value3".into()));

    let result = span_data.get_attribute("key_with_underscores").unwrap();
    assert_eq!(result, Value::String("value4".into()));

    let result = span_data.get_attribute("key@with@at@symbols").unwrap();
    assert_eq!(result, Value::String("value5".into()));

    let result = span_data.get_attribute("key/with/slashes").unwrap();
    assert_eq!(result, Value::String("value6".into()));

    let result = span_data.get_attribute("key:with:colons").unwrap();
    assert_eq!(result, Value::String("value7".into()));

    // Test that partial matches don't work
    let result = span_data.get_attribute("key-with");
    assert!(result.is_err());

    // Test that exact matching is required
    let result = span_data.get_attribute("key with spaces "); // Extra space at the end
    assert!(result.is_err());
}

#[test]
fn test_get_attribute_with_duplicate_keys() {
    let span_data = create_span_with_duplicate_keys();

    // Test that get_attribute returns the first occurrence of a duplicate key
    let result = span_data.get_attribute("duplicate_key").unwrap();
    assert_eq!(result, Value::String("first_value".into()));

    // Verify that unique keys still work correctly
    let result = span_data.get_attribute("unique_key").unwrap();
    assert_eq!(result, Value::String("unique_value".into()));

    let result = span_data.get_attribute("another_key").unwrap();
    assert_eq!(result, Value::String("another_value".into()));

    // Verify that get_attributes returns all attributes including duplicates
    let attributes = span_data.get_attributes();
    assert_eq!(attributes.len(), 5); // Should include all 5 attributes

    // Count occurrences of the duplicate key
    let duplicate_count = attributes
        .iter()
        .filter(|kv| kv.key.as_str() == "duplicate_key")
        .count();
    assert_eq!(duplicate_count, 3); // Should find all 3 occurrences

    // Verify the values of the duplicate keys in order
    let duplicate_values: Vec<&Value> = attributes
        .iter()
        .filter(|kv| kv.key.as_str() == "duplicate_key")
        .map(|kv| &kv.value)
        .collect();

    assert_eq!(duplicate_values.len(), 3);
    assert_eq!(*duplicate_values[0], Value::String("first_value".into()));
    assert_eq!(*duplicate_values[1], Value::String("second_value".into()));
    assert_eq!(*duplicate_values[2], Value::String("third_value".into()));
}

#[test]
fn test_get_attribute_with_empty_values() {
    let span_data = create_span_with_empty_values();

    // Test retrieving attributes with empty values
    let result = span_data.get_attribute("empty_string").unwrap();
    assert_eq!(result, Value::String("".into()));

    let result = span_data.get_attribute("zero_int").unwrap();
    assert_eq!(result, Value::I64(0));

    let result = span_data.get_attribute("zero_float").unwrap();
    assert_eq!(result, Value::F64(0.0));

    let result = span_data.get_attribute("false_bool").unwrap();
    assert_eq!(result, Value::Bool(false));

    let result = span_data.get_attribute("whitespace_string").unwrap();
    assert_eq!(result, Value::String("   ".into()));

    // Verify that normal attributes still work
    let result = span_data.get_attribute("normal_key").unwrap();
    assert_eq!(result, Value::String("normal_value".into()));

    // Verify that get_attributes returns all attributes including empty ones
    let attributes = span_data.get_attributes();
    assert_eq!(attributes.len(), 6); // Should include all 6 attributes

    // Verify that empty values are preserved in the attributes collection
    let empty_string_attr = attributes
        .iter()
        .find(|kv| kv.key.as_str() == "empty_string")
        .unwrap();
    assert_eq!(empty_string_attr.value, Value::String("".into()));

    let zero_int_attr = attributes
        .iter()
        .find(|kv| kv.key.as_str() == "zero_int")
        .unwrap();
    assert_eq!(zero_int_attr.value, Value::I64(0));
}
