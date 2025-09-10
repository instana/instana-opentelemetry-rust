# Instana Exporter for OpenTelemetry in Rust

Welcome to the documentation for the Instana exporter for OpenTelemetry in Rust. This library allows you to send OpenTelemetry trace data to Instana for monitoring and observability.

## Table of Contents

- [Getting Started](getting_started.md)
- [Exporter](exporter.md)
- [Propagation](propagation.md)
- [Serialization](serialization.md)
- [Examples](examples.md)
- [API Reference](api_reference.md)

## Overview

The Instana exporter for OpenTelemetry provides:

1. **Trace Export**: Send OpenTelemetry spans to Instana
2. **Context Propagation**: Propagate trace context using Instana headers
3. **Customization**: Configure the exporter to suit your needs

## Architecture

The library consists of several components:

- **Exporter**: Converts and sends spans to Instana
- **Propagator**: Handles context propagation using Instana headers
- **Serializer**: Converts OpenTelemetry spans to Instana's format
- **SpanData Extensions**: Utilities for working with span data

## Installation

Add the following to your `Cargo.toml`:

```toml
[dependencies]
opentelemetry_instana = "1.0.0"
```

## Basic Usage

```rust
use opentelemetry_instana::InstanaExporter;
use opentelemetry_instana::InstanaPropagator;
use opentelemetry::global;
use opentelemetry::propagation::TextMapPropagator;
use opentelemetry_sdk::trace::{self, Tracer};

// Create the Instana exporter using the builder pattern
let exporter = Exporter::builder()
    .build()
    .expect("Failed to create instana exporter");

// Create a batch span processor for the exporter
let batch_processor = trace::BatchSpanProcessor::builder(
    exporter,
    opentelemetry_sdk::runtime::Tokio,
).build();

// Create a provider with the batch processor
let provider = trace::TracerProvider::builder()
    .with_span_processor(batch_processor)
    .build();

// Set the global propagator to use Instana's propagator
let propagator = InstanaPropagator::new();
global::set_text_map_propagator(propagator);

// Set the global provider
let provider = global::set_tracer_provider(provider);

// Get a tracer from the provider
let tracer = provider.tracer("my-service");

// Create a span
let span = tracer.start("my-operation");
let _guard = span.enter();

// Your application code here
println!("Hello, world!");

// The span will be exported when it's dropped
```

## Configuration

The exporter can be configured using environment variables:

- `INSTANA_AGENT_HOST`: The hostname of the Instana agent (default: "localhost")
- `INSTANA_AGENT_PORT`: The port of the Instana agent (default: "42699")

Or programmatically:

```rust
let options = InstanaExporterOptions {
    endpoint: "http://custom-agent:42699/com.instana.plugin.generic.rawtrace".to_string(),
    hostname: "my-host".to_string(),
    service: "my-service".to_string(),
    ..Default::default()
};

let exporter = Exporter::builder()
    .with_options(options)
    .build()
    .expect("Failed to create instana exporter");
```

## License

Apache-2.0