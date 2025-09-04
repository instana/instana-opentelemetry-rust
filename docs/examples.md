# Working with Examples

This document provides guidance on how to set up, run, and create examples for the Instana Exporter for Rust.

## Available Examples

The project includes the following examples:

1. **Matrix Multiplication** - A web service that performs matrix multiplication and sends the result to the Matrix Printer service.
2. **Matrix Printer** - A web service that receives and displays matrix calculation results.

These examples demonstrate:
- Setting up the Instana Exporter
- Context propagation between services
- Span creation and attribute setting
- Distributed tracing across multiple services
- Cross-service and potentially cross-language tracing

## Running the Examples

### Prerequisites

- Rust and Cargo installed
- An Instana agent running locally or accessible via network

### Using the Run Script

The easiest way to run the examples is using the provided script:

```bash
cd examples
./run_servers.sh
```

This script:
1. Stops any existing processes on ports 8081 and 8083
2. Starts the Matrix Multiplication server on port 8081
3. Starts the Matrix Printer server on port 8083
4. Sets up proper cleanup when terminated

Once running, you can access:
- Matrix Multiplication UI: http://127.0.0.1:8081
- Matrix Printer UI: http://127.0.0.1:8083

### User Interface Features

**Matrix Multiplication UI:**
- Input fields for two 3x3 matrices
- "Multiply Matrices" button to perform calculation
- "Show Results in Matrix Printer" button to view results in the other service

**Matrix Printer UI:**
- Displays raw matrix data received from the multiplication service
- "Show Visual Matrix" button to display the matrix in a visual grid format

### Running Examples Individually

You can also run each example individually:

```bash
# Run Matrix Multiplication example
cd examples/matrix-multiplication
cargo run

# Run Matrix Printer example
cd examples/matrix-printer
cargo run
```

## Example Structure

Each example follows a similar structure:

1. **Initialization of the Instana Exporter**:
   ```rust
   let instana_exporter = Exporter::builder()
       .with_service(get_resource())
       .build()
       .expect("Failed to create instana exporter");
   let tracer_provider = opentelemetry_sdk::trace::SdkTracerProvider::builder()
       .with_batch_exporter(instana_exporter)
       .with_resource(get_resource())
       .build();
   global::set_tracer_provider(tracer_provider.clone());
   ```

2. **Resource Definition**:
   ```rust
   fn get_resource() -> Resource {
       static RESOURCE: OnceLock<Resource> = OnceLock::new();
       RESOURCE
           .get_or_init(|| {
               Resource::builder()
                   .with_service_name("your-service-name")
                   .with_attribute(KeyValue::new("process.pid", std::process::id() as i64))
                   .build()
           })
           .clone()
   }
   ```

3. **Creating Spans**:
   ```rust
   let tracer = global::tracer("tracer-name");
   let mut span = tracer.start("span-name");
   span.set_attribute(KeyValue::new("attribute.name", "value"));
   // ... perform operations ...
   span.end();
   ```

4. **Context Propagation**:
   ```rust
   // Inject context into headers (for outgoing requests)
   let propagator = Propagator::new();
   let mut injector: HashMap<String, String> = HashMap::new();
   let cx = Context::current_with_span(span);
   propagator.inject_context(&cx, &mut injector);
   
   // Extract context from headers (for incoming requests)
   let mut extractor: HashMap<String, String> = HashMap::new();
   for (key, value) in req.headers().iter() {
       if let Ok(val_str) = value.to_str() {
           extractor.insert(key.to_string(), val_str.to_string());
       }
   }
   let parent_context = propagator.extract(&extractor);
   ```

## Creating Your Own Examples

To create your own example:

1. Create a new directory under `examples/`
2. Create a new Cargo.toml file with the necessary dependencies:
   ```toml
   [dependencies]
   instana_opentelemetry_sdk = "1.0.0"
   opentelemetry = { version = "0.20.0", features = ["trace"] }
   opentelemetry_sdk = { version = "0.20.0", features = ["trace", "rt-tokio"] }
   ```

3. Create your application with the following key components:
   - Initialize the Instana exporter
   - Define resources with appropriate service name
   - Create spans for operations you want to trace
   - Add relevant attributes to spans
   - Implement context propagation for distributed tracing

4. Test your example with a local Instana agent

## Environment Variables

The examples use the following environment variables:

- `INSTANA_AGENT_HOST`: The hostname or IP address of the Instana agent (default: `127.0.0.1`)
- `INSTANA_AGENT_PORT`: The port of the Instana agent (default: `42699`)

You can set these variables before running the examples if your Instana agent is not running on the default host/port.

## Viewing Traces in Instana

After running the examples:

1. Open your Instana dashboard
2. Navigate to the "Analyze > Traces" section
3. Look for traces from services named "matrix-multplier_propgation" and "matrix-printer_using_propgated_context"
4. Click on a trace to view the detailed span information

This will show you the complete distributed trace across both services, including all the custom attributes set in the examples.


