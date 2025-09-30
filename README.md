# Instana OpenTelemetry Rust

## Overview

Instana OpenTelemetry Rust is based on Open Source [OpenTelemetry Rust](https://github.com/open-telemetry/opentelemetry-rust). It provides an OpenTelemetry implementation which focuses on supporting Instana OpenTelemetry and IBM platforms (S390X Linux, PowerPC Linux, AIX) as well as other platforms (Linux x64/ARM64 and Windows).

It includes standard features which provide a collection of tools, APIs, and SDKs used to instrument, generate, collect, and export telemetry data (metrics, logs, and traces) for analysis in order to understand your software's performance and behavior.

In addition to standard features of OpenTelemetry Rust, Instana OpenTelemetry Rust also provides Instana OpenTelemetry Rust SDK which supports exporter, propagation and serialization. It allows you to send OpenTelemetry trace data to Instana for monitoring and observability. See [Instana Opentelemetry Rust SDK](./opentelemetry-instana/README.md) for more details.

## Project Status

The table below summarizes the overall status of each component. Some components
include unstable features, which are documented in their respective crate
documentation.

| Signal/Component      | Overall Status     |
| --------------------  | ------------------ |
| Context               | Beta               |
| Baggage               | RC                 |
| Propagators           | Beta               |
| Logs-API              | Stable*            |
| Logs-SDK              | Stable             |
| Logs-OTLP Exporter    | RC                 |
| Logs-Appender-Tracing | Stable             |
| Metrics-API           | Stable             |
| Metrics-SDK           | Stable             |
| Metrics-OTLP Exporter | RC                 |
| Traces-API            | Beta               |
| Traces-SDK            | Beta               |
| Traces-OTLP Exporter  | Beta               |

*OpenTelemetry Rust is not introducing a new end user callable Logging API.
Instead, it provides [Logs Bridge
API](https://github.com/open-telemetry/opentelemetry-specification/blob/main/specification/logs/api.md),
that allows one to write log appenders that can bridge existing logging
libraries to the OpenTelemetry log data model. The following log appenders are
available:

* [opentelemetry-appender-log](opentelemetry-appender-log/README.md)
* [opentelemetry-appender-tracing](opentelemetry-appender-tracing/README.md)

If you already use the logging APIs from above, continue to use them, and use
the appenders above to bridge the logs to OpenTelemetry. If you are using a
library not listed here, feel free to contribute a new appender for the same.

If you are starting fresh, we recommend using
[tracing](https://github.com/tokio-rs/tracing) as your logging API. It supports
structured logging and is actively maintained. `OpenTelemetry` itself uses
`tracing` for its internal logging.


## Getting Started

### Download and Build
The Instana OpenTelemetry Rust is available in source as `tar.gz` or `zip` file which can be downloaded from [releases](https://github.com/instana/instana-opentelemetry-rust/releases).

Before building from source, make sure the following tools are installed:
 - Rust 1.86 or above (Note: Only Rust 1.86 is tested on AIX)
 - gcc/g++ 13.0.0 (or above) (Linux, AIX), set environment variable `CC` to `gcc` and `CXX` to `g++`
 - openssl develop package (Linux, AIX), set environment variable `OPENSSL_DIR` and `OPENSSL_LIB_DIR` to point to the openssl installation directory and openssl library directory respectively.

To build everything, run:
```
cargo build --release
```

### Using the crates

There are two ways to use the crates:

1. Use path to the crate in your `Cargo.toml`:
for example:
```
[dependencies]
<crate-name> = { path = "<path-to-the-root-of-source>" } 
```

2. Install the crates locally by using `cargo install`:
```

cargo install --path <path-to-the-root-of-source>
```

### Examples provided in opentelemetry-instana

The examples in opentelemetry-instana folder show how to use instana opentelemetry exporter to send trace data to Instana. See details in [examples/README.md](./opentelemetry-instana/docs/examples.md).

### Examples for using IDOT (Instana Distribution of OpenTelemetry Collector)

Examples are also provided for sending trace data through IDOT (Instana Distribution of OpenTelemetry Collector) with OpenTelemetry Rust SDK. See [base-otlp on IDOT](./opentelemetry-otlp/examples/basic-otlp/README.md) and [basic-otlp-http on IDOT](./opentelemetry-otlp/examples/basic-otlp-http/README.md) for more details.

## Overview of crates

The following crates are maintained in this repo:

* [`opentelemetry-instana`] This is the Instana Opentelemetry Rust SDK
   which supports exporter, propagation and serialization. It allows you to send
   OpenTelemetry trace data to Instana for monitoring and observability
* [`opentelemetry`] This is the OpenTelemetry API crate, and is the crate
  required to instrument libraries and applications. It contains Context API,
  Baggage API, Propagators API, Logging Bridge API, Metrics API, and Tracing
  API.
* [`opentelemetry-sdk`] This is the OpenTelemetry SDK crate, and contains the
  official OpenTelemetry SDK implementation. It contains Logging SDK, Metrics
  SDK, and Tracing SDK. It also contains propagator implementations.
* [`opentelemetry-otlp`] - exporter to send telemetry (logs, metrics and traces)
  in the [OTLP
  format](https://github.com/open-telemetry/opentelemetry-specification/tree/main/specification/protocol)
  to an endpoint accepting OTLP. This could be the [OTel
  Collector](https://github.com/open-telemetry/opentelemetry-collector),
  telemetry backends like [Jaeger](https://www.jaegertracing.io/),
* [`opentelemetry-stdout`] exporter for sending logs, metrics and traces to
  stdout, for learning/debugging purposes.  
* [`opentelemetry-http`] This crate contains utility functions to help with
  exporting telemetry, propagation, over [`http`].
* [`opentelemetry-appender-log`] This crate provides logging appender to route
  logs emitted using the [log](https://docs.rs/log/latest/log/) crate to
  opentelemetry.
* [`opentelemetry-appender-tracing`] This crate provides logging appender to
  route logs emitted using the [tracing](https://crates.io/crates/tracing) crate
  to opentelemetry.  
* [`opentelemetry-semantic-conventions`] provides standard names and semantic
  otel conventions.

### Thanks to all the people who have contributed

[![contributors](https://contributors-img.web.app/image?repo=open-telemetry/opentelemetry-rust)](https://github.com/open-telemetry/opentelemetry-rust/graphs/contributors)
