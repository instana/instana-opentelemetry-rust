# Architecture and Design

This document describes the architecture and design of the Instana exporter for OpenTelemetry in Rust.

## Overview

The Instana exporter for OpenTelemetry is designed to bridge the gap between OpenTelemetry instrumentation and Instana's monitoring system. It follows a modular architecture with clear separation of concerns between exporting, propagation, and serialization components.

```
┌───────────────────────────────────────────────────────────────┐
│                        Application Code                       │
└───────────────────────────────┬───────────────────────────────┘
                                │
                                ▼
┌───────────────────────────────────────────────────────────────┐
│                   OpenTelemetry SDK & API                     │
└───────────────────────────────┬───────────────────────────────┘
                                │
                                ▼
┌───────────────────────────────────────────────────────────────┐
│                        Instana Exporter                       │
│                                                               │
│  ┌─────────────────┐  ┌──────────────┐  ┌──────────────────┐  │
│  │                 │  │              │  │ Serialization    │  │
│  │    Exporter     │  │  Propagator  │  │                  │  │
│  │                 │  │              │  │ HttpBodyWrapper  │  │
│  │                 │  │              │  │ serialize_span   │  │
│  └────────┬────────┘  └───────┬──────┘  └────────┬─────────┘  │
│           │                   │                  │            │
└───────────┼───────────────────┼──────────────────┼────────────┘
            │                   │                  │
            ▼                   ▼                  ▼
  ┌───────────────────┐ ┌────────────────┐ ┌─────────────────┐
  │   Instana Agent   │ │  HTTP Headers  │ │   JSON Format   │
  └───────────────────┘ └────────────────┘ └─────────────────┘
```

## Core Components

### 1. Exporter Module

The exporter module is responsible for sending OpenTelemetry spans to the Instana backend.

**Key Components:**
- `InstanaExporter`: Implements the OpenTelemetry `SpanExporter` trait
- `InstanaExporterOptions`: Configuration options for the exporter
- `Builder`: Builder pattern for creating exporters

**Design Principles:**
- Follows the OpenTelemetry SDK's exporter interface
- Uses a builder pattern for flexible configuration
- Supports environment variable configuration
- Handles HTTP communication with the Instana agent

### 2. Propagation Module

The propagation module handles context propagation using Instana's HTTP headers.

**Key Components:**
- `Propagator`: Implements the OpenTelemetry `TextMapPropagator` trait

**Design Principles:**
- Bidirectional conversion between OpenTelemetry context and Instana headers
- Maintains compatibility with Instana's trace correlation format
- Handles validation and error cases gracefully

### 3. Serialization Components

The serialization components convert OpenTelemetry spans to Instana's JSON format.

**Key Components:**
- `HttpBodyWrapper`: Wrapper around `BytesMut` for building JSON payloads
- `serialize_span`: Function to convert spans to Instana's format
- `GET` trait: Extensions for accessing span data

**Design Principles:**
- Efficient memory management with BytesMut
- Structured approach to building JSON payloads
- Clear mapping between OpenTelemetry and Instana concepts

## Data Flow

### Export Flow

1. The application creates spans using OpenTelemetry API
2. Spans are processed by the OpenTelemetry SDK
3. The SDK passes spans to the `Exporter`
4. The exporter uses `serialize_span` to convert spans to Instana's format
5. The serialized spans are sent to the Instana agent via HTTP

```
Application → OpenTelemetry SDK → Exporter → serialize_span → HTTP Request → Instana Agent
```

### Propagation Flow

#### Injection
1. The application creates a span with OpenTelemetry
2. When making an outgoing request, the application calls the propagator's `inject_context`
3. The `Propagator` converts the span context to Instana headers
4. The headers are added to the outgoing request

```
SpanContext → Propagator.inject_context → Instana Headers → Outgoing Request
```

#### Extraction
1. The application receives an incoming request with Instana headers
2. The application calls the propagator's `extract_with_context`
3. The `InstanaPropagator` converts the headers to a span context
4. The application creates a new span with the extracted context as parent

```
Incoming Request → Instana Headers → InstanaPropagator.extract_with_context → SpanContext
```

## Serialization Format

The exporter converts OpenTelemetry spans to Instana's JSON format:

```json
[
  {
    "t": "trace_id_16_chars",
    "s": "span_id_16_chars",
    "p": "parent_span_id_16_chars",
    "ts": timestamp_in_ms,
    "d": duration_in_ms,
    "n": "sdk",
    "k": 1,
    "data": {
      "sdk": {
        "name": "span_name",
        "type": "entry|exit|intermediate",
        "custom": {
          "tags": {
            "attributes": { ... },
            "resource": { ... },
            "events": { ... },
            "links": [ ... ],
            "otel": { ... }
          }
        }
      },
      "service": "service_name"
    },
    "f": {
      "e": "process_id",
      "h": "host_id"
    }
  }
]
```

## Design Decisions

### 1. Separation of Concerns

The library is divided into distinct modules with clear responsibilities:
- InstanaExporter: Handles communication with Instana
- InstanaPropagator: Handles context propagation
- Serialization: Handles data format conversion

This separation makes the code more maintainable and testable.

### 2. OpenTelemetry Compatibility

The library strictly adheres to OpenTelemetry interfaces:
- Implements `SpanExporter` for exporting spans
- Implements `TextMapPropagator` for context propagation
- Uses OpenTelemetry data types throughout

This ensures compatibility with the broader OpenTelemetry ecosystem.

### 3. Efficient Serialization

The serialization process uses `BytesMut` for efficient memory management:
- Avoids unnecessary allocations
- Provides direct access to the underlying buffer
- Allows for incremental building of JSON payloads

### 4. Configuration Flexibility

The library supports multiple configuration methods:
- Environment variables for simple deployment
- Programmatic configuration for fine-grained control
- Builder pattern for readable code

### 5. Error Handling

The library uses a combination of Rust's Result type and OpenTelemetry's error types:
- Returns `OTelSdkResult` from exporter methods
- Uses custom error types for internal errors
- Provides meaningful error messages