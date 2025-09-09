use instana_opentelemetry_sdk::exporter::{exporter::Exporter, serialize_span};
use opentelemetry::trace::{SpanContext, SpanId, SpanKind, Status};
use opentelemetry::{InstrumentationScope, KeyValue};
use opentelemetry_sdk::{
    trace::{SpanData, SpanEvents, SpanLinks},
    Resource,
};
use serde_json::json;
use std::sync::OnceLock;
use std::time::SystemTime;

fn get_resource() -> Resource {
    static RESOURCE: OnceLock<Resource> = OnceLock::new();
    RESOURCE
        .get_or_init(|| {
            Resource::builder()
                .with_service_name("basic-instana-test")
                .build()
        })
        .clone()
}
#[test]
fn test_serialize() {
    let mut vector: Vec<KeyValue> = Vec::new();
    vector.push(KeyValue::new("INTERNAL_TAG_ENTITY_ID", 100));
    vector.push(KeyValue::new("INTERNAL_TAG_CRTP", "ntg"));
    let exporter = Exporter::builder()
        .with_service(get_resource())
        .build()
        .unwrap();
    let time = SystemTime::now();
    let span_data = SpanData {
        span_context: SpanContext::NONE,
        parent_span_id: SpanId::INVALID,
        span_kind: SpanKind::Client,
        name: std::borrow::Cow::Borrowed("testspan"),
        start_time: time,
        end_time: time,
        attributes: vector,
        dropped_attributes_count: 100,
        events: SpanEvents::default(),
        links: SpanLinks::default(),
        status: Status::Ok,
        instrumentation_scope: InstrumentationScope::builder("fake-instrumentation").build(),
    };

    let instana_span = serialize_span::convert_to_instana_span(&exporter, &span_data).unwrap();

    let converted_json = serde_json::to_value(&instana_span).unwrap();

    let timestamp = span_data
        .start_time
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap()
        .as_millis();
    let expected_json = json!(
        {
            "crtp": "ntg",
            "d": 0,
            "data": {
                "sdk": {
                    "custom": {
                        "tags": {
                            "attributes": {
                                "INTERNAL_TAG_CRTP": "ntg",
                                "INTERNAL_TAG_ENTITY_ID": 100
                            },
                            "otel": {
                                "dropped_attributes_count": 100,
                                "dropped_events_count": 0,
                                "dropped_links_count": 0,
                                "scope.name": "fake-instrumentation",
                                "status_code": "StatusCode::STATUS_OK"
                            },
                            "resource": {
                                "telemetry.sdk.language": "rust",
                                "telemetry.sdk.name": "opentelemetry",
                                "telemetry.sdk.version": "0.29.0"
                            }
                        }
                    },
                    "name": "testspan",
                    "type": "exit"
                },
                "service": "basic-instana-test"
            },
            "f": {},
            "k": 2,
            "n": "sdk",
            "s": "0000000000000000",
            "sy": false,
            "t": "0000000000000000",
            "ts": timestamp as i64
        }

    );

    assert_eq!(converted_json, expected_json);
}
