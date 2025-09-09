use instana_opentelemetry_sdk::exporter::exporter::{Exporter, Options};
use opentelemetry::trace::{
    SpanContext, SpanId, SpanKind, Status, TraceFlags, TraceId, TraceState,
};
use opentelemetry::{InstrumentationScope, KeyValue};
use opentelemetry_http::HttpClient;
use opentelemetry_sdk::error::OTelSdkError;
use opentelemetry_sdk::trace::{SpanData, SpanEvents, SpanExporter, SpanLinks};
use opentelemetry_sdk::Resource;
use reqwest::header::{HeaderValue, CONTENT_TYPE};
use std::sync::{Arc, OnceLock};
use std::time::SystemTime;
use tokio;
use wiremock::{
    matchers::{method, path},
    Mock, MockServer, ResponseTemplate,
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

fn create_test_span_data() -> SpanData {
    let mut attributes: Vec<KeyValue> = Vec::new();
    attributes.push(KeyValue::new("INTERNAL_TAG_ENTITY_ID", 100));
    attributes.push(KeyValue::new("INTERNAL_TAG_CRTP", "ntg"));

    let time = SystemTime::now();

    SpanData {
        span_context: SpanContext::new(
            TraceId::from_hex("0102030405060708090a0b0c0d0e0f10").unwrap(),
            SpanId::from_hex("0102030405060708").unwrap(),
            TraceFlags::SAMPLED,
            false,
            TraceState::default(),
        ),
        parent_span_id: SpanId::INVALID,
        span_kind: SpanKind::Client,
        name: std::borrow::Cow::Borrowed("test-span"),
        start_time: time,
        end_time: time,
        attributes,
        dropped_attributes_count: 0,
        events: SpanEvents::default(),
        links: SpanLinks::default(),
        status: Status::Ok,
        instrumentation_scope: InstrumentationScope::builder("test-instrumentation").build(),
    }
}

#[test]
fn test_instana_exporter_options_default() {
    let options = Options::default();

    // Test that default values are set correctly
    assert!(options
        .endpoint
        .contains("/com.instana.plugin.generic.rawtrace"));
    assert_eq!(options.hostname, String::new());
    assert_eq!(options.source_address, String::new());
    assert_eq!(options.service, String::new());

    // Test that headers are set correctly
    assert!(options.headers.contains_key(CONTENT_TYPE));
    assert_eq!(
        options.headers.get(CONTENT_TYPE).unwrap(),
        &HeaderValue::from_static("application/json")
    );
}

#[test]
fn test_instana_exporter_options_with_endpoint() {
    let endpoint = "http://test-endpoint:1234/path";
    let options = Options::with_endpoint(endpoint).unwrap();

    assert_eq!(options.endpoint, endpoint);
    assert_eq!(options.hostname, String::new());
    assert_eq!(options.source_address, String::new());
    assert_eq!(options.service, String::new());
}

#[test]
fn test_instana_exporter_options_with_invalid_endpoint() {
    let invalid_endpoint = "not-a-valid-url";
    let options = Options::with_endpoint(invalid_endpoint);

    assert!(options.is_none());
}

#[test]
fn test_instana_exporter_builder() {
    let resource = get_resource();
    let options = Options::default();

    let exporter = Exporter::builder()
        .with_service(resource.clone())
        .with_options(options.clone())
        .build()
        .expect("failed to build instana exporter");

    assert_eq!(exporter.get_options(), options);
    assert_eq!(exporter.get_resource(), resource);
}

#[test]
fn test_instana_exporter_builder_with_http_client() {
    let client = reqwest::blocking::Client::builder()
        .build()
        .unwrap_or_default();
    let resource = get_resource();
    let options = Options::default();

    let exporter = Exporter::builder()
        .with_service(resource.clone())
        .with_options(options.clone())
        .with_http_client(client)
        .build()
        .expect("failed to build instana exporter");

    assert_eq!(exporter.get_options(), options);
    assert_eq!(exporter.get_resource(), resource);
}

#[test]
fn test_instana_exporter_resource_getters() {
    let exporter = Exporter::builder()
        .with_service(get_resource())
        .build()
        .expect("failed to build instana exporter");

    // Test service name getter
    let service_name = exporter.get_service_name().unwrap();
    assert_eq!(service_name, "test-service".into());

    // Test process pid getter
    let process_pid = exporter.get_process_pid();
    assert!(process_pid.is_none()); // We didn't set process.pid in this test

    // Test host id getter
    let host_id = exporter.get_host_id();
    assert!(host_id.is_none()); // We didn't set host.id in this test

    // Test cloud provider getter
    let cloud_provider = exporter.get_cloud_provider();
    assert!(cloud_provider.is_none()); // We didn't set cloud.provider in this test
}

#[tokio::test]
async fn test_instana_exporter_export_success() {
    // Start a mock server
    let mock_server = MockServer::start().await;

    // Configure the mock server to respond with success
    Mock::given(method("POST"))
        .and(path("/test-path"))
        .respond_with(ResponseTemplate::new(200).set_body_string("Success"))
        .mount(&mock_server)
        .await;

    // Create exporter with the mock server URL
    let options = Options::with_endpoint(&format!("{}/test-path", mock_server.uri())).unwrap();

    let client: Arc<dyn HttpClient> =
        Arc::new(reqwest::Client::builder().build().unwrap_or_default());

    let exporter = Exporter::new(client, options, get_resource());

    // Create a batch of spans to export
    let batch = vec![create_test_span_data()];

    // Export the spans
    let result = exporter.export(batch).await;

    // Verify the export was successful
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_instana_exporter_export_server_error() {
    // Start a mock server
    let mock_server = MockServer::start().await;

    // Configure the mock server to respond with an error
    Mock::given(method("POST"))
        .and(path("/test-path"))
        .respond_with(ResponseTemplate::new(500).set_body_string("Server Error"))
        .mount(&mock_server)
        .await;

    // Create exporter with the mock server URL
    let options = Options::with_endpoint(&format!("{}/test-path", mock_server.uri())).unwrap();

    let client: Arc<dyn HttpClient> =
        Arc::new(reqwest::Client::builder().build().unwrap_or_default());

    let exporter = Exporter::new(client, options, get_resource());

    // Create a batch of spans to export
    let batch = vec![create_test_span_data()];

    // Export the spans
    let result = exporter.export(batch).await;

    // Verify the export failed with the expected error
    assert!(result.is_err());
    match result {
        Err(OTelSdkError::InternalFailure(msg)) => {
            // Just check for the status code since the error message might vary
            assert!(msg.contains("500"));
        },
        _ => panic!("Expected InternalFailure error"),
    }
}

#[test]
fn test_instana_exporter_shutdown() {
    let client: Arc<dyn HttpClient> = Arc::new(
        reqwest::blocking::Client::builder()
            .build()
            .unwrap_or_default(),
    );

    let options = Options::default();
    let mut exporter = Exporter::new(client, options, get_resource());

    // First shutdown should succeed
    let result = exporter.shutdown();
    assert!(result.is_ok());

    // Second shutdown should fail with AlreadyShutdown
    let result = exporter.shutdown();
    assert!(matches!(result, Err(OTelSdkError::AlreadyShutdown)));
}

#[test]
fn test_instana_exporter_force_flush() {
    let client: Arc<dyn HttpClient> = Arc::new(
        reqwest::blocking::Client::builder()
            .build()
            .unwrap_or_default(),
    );

    let options = Options::default();
    let mut exporter = Exporter::new(client, options, get_resource());

    // Force flush should always succeed
    let result = exporter.force_flush();
    assert!(result.is_ok());
}

#[test]
fn test_instana_exporter_set_resource() {
    let client: Arc<dyn HttpClient> = Arc::new(
        reqwest::blocking::Client::builder()
            .build()
            .unwrap_or_default(),
    );

    let options = Options::default();
    let base_resource = Resource::builder().build();
    let mut exporter = Exporter::new(client, options, base_resource.clone());

    // Initially the resource should be the base resource
    assert_eq!(exporter.get_resource(), base_resource);

    // Set a new resource
    let new_resource = get_resource();
    exporter.set_resource(&new_resource);

    // Verify the resource was updated
    assert_eq!(exporter.get_resource(), new_resource);
}
