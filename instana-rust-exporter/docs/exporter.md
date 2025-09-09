# OpenTelemetry Instana Exporter

The OpenTelemetry Exporter for Instana provides a way to export OpenTelemetry spans to IBM Observability by Instana. This exporter transforms OpenTelemetry `SpanData` into the Instana trace format and sends it to the specified endpoint.

## Features

- Converts OpenTelemetry spans to Instana's trace format
- Automatically detects Instana agent using environment variables
- Supports span attributes, events, and links
- Preserves OpenTelemetry instrumentation scope information
- Handles different span kinds (entry, exit, intermediate)
- Supports resource attributes for service identification
- Uses `reqwest::blocking::client` for HTTP communication with the Instana backend

## Usage

### Basic Usage

```rust
use opentelemetry_sdk::trace::TracerProvider;
use opentelemetry_sdk::export::trace::BatchSpanProcessor;
use crate::exporter::exporter::Exporter;

// Create the Instana exporter using the builder pattern
let exporter = Exporter::builder()
    .build()
    .expect("Failed to create instana exporter");

// Create a batch span processor that uses the Instana exporter
let batch_processor = BatchSpanProcessor::builder(exporter, opentelemetry_sdk::runtime::Tokio)
    .build();

// Create a tracer provider with the batch processor
let provider = TracerProvider::builder()
    .with_span_processor(batch_processor)
    .build();

// Set the provider as the global tracer provider
opentelemetry::global::set_tracer_provider(provider);
```

### Custom Configuration

```rust
use opentelemetry_sdk::Resource;
use opentelemetry::KeyValue;
use crate::exporter::exporter::{Exporter, Options};

// Create custom exporter options
let options = Options {
    endpoint: "http://custom-agent:42699/com.instana.plugin.generic.rawtrace".to_string(),
    hostname: "my-host".to_string(),
    service: "my-service".to_string(),
    ..Default::default()
};

// Create a resource with service information
let resource = Resource::new(vec![
    KeyValue::new("service.name", "my-application"),
    KeyValue::new("service.version", "1.0.0"),
    KeyValue::new("host.id", "unique-host-id"),
]);

// Create the exporter with custom options and resource
let exporter = Exporter::builder()
    .with_options(options)
    .with_service(resource)
    .build();
```

## Configuration Options

The `Options` struct provides the following configuration options:

- `endpoint`: The URL endpoint of the Instana agent (default: `http://{INSTANA_AGENT_HOST}:{INSTANA_AGENT_PORT}/com.instana.plugin.generic.rawtrace`)
- `hostname`: The hostname to report to Instana
- `source_address`: The source address to report to Instana
- `service`: The service name to report to Instana
- `headers`: Additional HTTP headers to include in requests to the Instana agent

## Environment Variables

The exporter uses the following environment variables:

- `INSTANA_AGENT_HOST`: The hostname or IP address of the Instana agent (default: `127.0.0.1`)
- `INSTANA_AGENT_PORT`: The port of the Instana agent (default: `42699`)

## Span Data Mapping

The exporter maps OpenTelemetry span data to Instana's trace format:

- OpenTelemetry `SpanContext.trace_id` → Instana `t` (trace ID)
- OpenTelemetry `SpanContext.span_id` → Instana `s` (span ID)
- OpenTelemetry `parent_span_id` → Instana `p` (parent span ID)
- OpenTelemetry `SpanKind` → Instana `k` (kind)
  - `SpanKind::Server` and `SpanKind::Producer` → `1` (entry)
  - `SpanKind::Client` and `SpanKind::Consumer` → `2` (exit)
  - `SpanKind::Internal` → `3` (intermediate)
- OpenTelemetry `start_time` → Instana `ts` (timestamp)
- OpenTelemetry `end_time - start_time` → Instana `d` (duration)
- OpenTelemetry `name` → Instana `sdk.name`
- OpenTelemetry `attributes` → Instana `sdk.custom.tags.attributes`
- OpenTelemetry `events` → Instana `sdk.custom.tags.events`
- OpenTelemetry `links` → Instana `sdk.custom.tags.links`
- OpenTelemetry `status` → Instana `ec` (error count) and `sdk.custom.tags.otel.status_code`
- OpenTelemetry `instrumentation_scope` → Instana `sdk.custom.tags.otel.scope.name` and `sdk.custom.tags.otel.scope.version`

## Useful Links

- For more information on Instana, visit [Instana's website](https://www.instana.com/) and [Instana's documentation](https://www.ibm.com/docs/en/obi/current)
- For more information on OpenTelemetry, visit [opentelemetry.io](https://opentelemetry.io/)


