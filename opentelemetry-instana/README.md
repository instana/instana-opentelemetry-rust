# Instana Opentelemetry Rust Exporter

This library provides an Instana Exporter for OpenTelemetry which supports exporter, propagation and serialization. It allows you to send OpenTelemetry trace data to Instana for monitoring and observability.

## Features

- Export OpenTelemetry spans to Instana
- Propagate trace context using Instana headers
- Customize exporter configuration
- Support for resource attributes
- Composite propagator support for multi-format propagation
- Working examples demonstrating distributed tracing

## Installation

Add the following to your `Cargo.toml`:

```toml
[dependencies]
opentelemetry_instana = "1.0.0"
```

## Usage

### Exporter

```rust
use opentelemetry_instana::{InstanaExporter,InstanaExporterOptions};
use opentelemetry_sdk::trace::SpanExporter;
use opentelemetry_sdk::Resource;
use opentelemetry::KeyValue;

// Create exporter with builder pattern (recommended approach)
let exporter = InstanaExporter::builder()
    .build()
    .expect("Failed to create instana exporter");

// Create with custom options
let options = InstanaExporterOptions {
    endpoint: format!(
                "http://{}:{}/com.instana.plugin.generic.rawtrace",
                host, port
            ),
    hostname: "my-host".to_string(),
    service: "my-service".to_string(),
    ..Default::default()
};

// Create exporter with custom options and service
let exporter = InstanaExporter::builder()
    .with_options(options)
    .with_service(Resource::new(vec![KeyValue::new("service.name", "my-service")]))
    .build()
    .expect("Failed to create instana exporter");
```

### Propagator

#### Basic Instana Propagator

```rust
use opentelemetry_instana::InstanaPropagator;
use opentelemetry::global;
use opentelemetry::propagation::TextMapPropagator;

// Create and register the Instana propagator
let propagator = InstanaPropagator::new();
global::set_text_map_propagator(propagator);
```

#### Composite Propagator

The composite propagator allows you to use multiple propagation formats simultaneously:

```rust
use opentelemetry_instana::composite::CompositePropagator;
use opentelemetry_instana::InstanaPropagator;
use opentelemetry::global;
use opentelemetry::propagation::{TextMapPropagator, TraceContextPropagator};

// Create a composite propagator with both Instana and W3C TraceContext formats
let propagators: Vec<Box<dyn TextMapPropagator + Send + Sync>> = vec![
    Box::new(InstanaPropagator::new()),
    Box::new(TraceContextPropagator::new()),
];

let composite_propagator = CompositePropagator::new(propagators);
global::set_text_map_propagator(composite_propagator);
```

## Configuration

### Environment Variables

The exporter uses the following environment variables:

- `INSTANA_AGENT_HOST`: The hostname of the Instana agent (default: "localhost")
- `INSTANA_AGENT_PORT`: The port of the Instana agent (default: "42699")

### Options

The `InstanaExporterOptions` struct provides configuration options for `InstanaExporter`:

| Field | Description | Default |
|-------|-------------|---------|
| `endpoint` | The URL endpoint for the Instana agent | `http://{INSTANA_AGENT_HOST}:{INSTANA_AGENT_PORT}/com.instana.plugin.generic.rawtrace` |
| `hostname` | The hostname to report | Empty string |
| `source_address` | The source address to report | Empty string |
| `service` | The service name to report | Empty string |
| `headers` | Additional HTTP headers to include in requests | `Content-Type: application/json` |

## Propagation

### Instana Headers

The Instana propagator handles the following HTTP headers:

- `X-INSTANA-T`: The trace ID (16 or 32 hex characters)
- `X-INSTANA-S`: The parent span ID (16 hex characters)
- `X-INSTANA-L`: The sampling level (0 or 1)

These headers are translated to and from the OpenTelemetry `SpanContext`.

### Header Details

- **X-INSTANA-T (trace ID)**: A string of either 16 or 32 characters from the alphabet `0-9a-f`, representing either a 64 bit or 128 bit ID. If the propagator receives a value shorter than 32 characters when extracting headers, it will left-pad the string with "0" to length 32.

- **X-INSTANA-S (parent span ID)**: A string of 16 characters from the alphabet `0-9a-f`, representing a 64 bit ID.

- **X-INSTANA-L (sampling level)**: The only valid values are `1` (sampled) and `0` (not sampled).

## Examples

The project includes working examples that demonstrate how to use the Instana exporter in real-world scenarios:

### Matrix Multiplication and Matrix Printer

Two interconnected services that demonstrate distributed tracing:
- Matrix Multiplication service performs calculations and sends results to the Matrix Printer
- Matrix Printer service receives and displays the results
- Trace context is propagated between services using Instana headers

To run the examples:

```bash
cd examples
./run_servers.sh
```

Then visit:
- Matrix Multiplication UI: http://127.0.0.1:8081
- Matrix Printer UI: http://127.0.0.1:8083

For more details, see the [Examples Documentation](docs/examples.md).

## Architecture

### Components

- **InstanaExporter**: Sends spans to Instana
- **InstanaPropagator**: Handles context propagation using Instana headers
- **CompositePropagator**: Allows using multiple propagation formats simultaneously
- **Serializer**: Converts OpenTelemetry spans to Instana's format
- **SpanData**: Extensions for working with OpenTelemetry span data

### Serialization

The exporter includes several components for serializing OpenTelemetry spans to Instana's format:

- **HttpBodyWrapper**: Provides methods for building JSON payloads
- **serialize_span**: Converts OpenTelemetry spans to Instana's format
- **SpanData Extensions**: Extends `SpanData` with methods for accessing span data

## License

Apache-2.0
