# Serialization Components

The Instana exporter includes several components for serializing OpenTelemetry spans to Instana's format.

## InstanaSpan Structure

The serialization process converts OpenTelemetry spans to the `InstanaSpan` structure, which is then serialized to JSON. The structure includes:

### Core Fields
- `parent_span_id` (serialized as `p`): Parent span ID (16 hex chars)
- `trace_id` (serialized as `t`): Trace ID (16 hex chars, right-most part)
- `span_id` (serialized as `s`): Span ID (16 hex chars)
- `name` (serialized as `n`): Name (always "sdk" for SDK spans)
- `kind` (serialized as `k`): Kind (1=entry, 2=exit, 3=intermediate)
- `timestamp` (serialized as `ts`): Timestamp in milliseconds
- `duration` (serialized as `d`): Duration in milliseconds
- `synthetic` (serialized as `sy`): Synthetic flag

### Optional Fields
- `long_trace_id` (serialized as `lt`): Long trace ID (32 hex chars, only for entry spans)
- `error_count` (serialized as `ec`): Error count
- `correlation_id` (serialized as `crid`): Correlation ID
- `correlation_type` (serialized as `crtp`): Correlation type
- `trace_parent` (serialized as `tp`): Trace parent flag

### Data Section
The `data` field contains span details in the `InstanaSpanData` structure:
- `sdk`: Contains span name, type, and custom data
- `service`: Service name

### From Section
The `from` field (serialized as `f`) contains information about the span source:
- `process_id` (serialized as `e`): Process ID
- `host_id` (serialized as `h`): Host ID

## Additional Structures

### InstanaLink Structure
The `InstanaLink` structure represents links between spans:
- `trace_id` (serialized as `t`): Trace ID (16 hex chars, right-most part)
- `span_id` (serialized as `s`): Span ID (16 hex chars)
- `attributes`: Map of link attributes

## Serialization Process

The serialization process is handled by the `serialize_span` module, which provides functions to convert OpenTelemetry spans to Instana's format.

### Main Functions

- `convert_to_instana_span`: Converts an OpenTelemetry `SpanData` to an `InstanaSpan`
- `serialize_batch`: Serializes a batch of spans to JSON bytes
- `build_data_section`: Builds the data section of the span
- `build_from_section`: Builds the from section of the span
- `extract_correlation_data`: Extracts correlation ID and type from span

### Conversion Process

1. **Extract span context information**
   - Trace ID, span ID, parent span ID
   - Determine span kind (entry, exit, intermediate)

2. **Process timing information**
   - Start time in milliseconds
   - Duration in milliseconds

3. **Handle special fields**
   - Long trace ID (for entry spans)
   - Synthetic flag
   - Error count
   - Correlation data
   - Trace parent flag

4. **Build data section**
   - Convert span attributes
   - Convert resource attributes
   - Convert events and links
   - Process status information
   - Build OTEL section with instrumentation scope details

5. **Build from section**
   - Process ID
   - Host ID

6. **Serialize to JSON**
   - Convert the `InstanaSpan` structure to JSON using serde

### Span Kind Mapping

OpenTelemetry span kinds are mapped to Instana span types:

- `SpanKind::Server` and `SpanKind::Producer` → "entry" (k=1)
- `SpanKind::Client` and `SpanKind::Consumer` → "exit" (k=2)
- `SpanKind::Internal` → "intermediate" (k=3)

## SpanData Extensions

The `GET` trait extends `SpanData` with methods for accessing span data:

- `get_attribute`: Gets a specific attribute by key
- `get_attributes`: Gets all attributes
- `get_events`: Gets all events
- `get_links`: Gets all links

## Constants

The `defs` module defines constants used in serialization:

- Internal tag names
- Default configuration values
- OpenTelemetry key names