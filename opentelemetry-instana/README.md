---

copyright:
  years: [{CURRENT_YEAR}]
lastupdated: [{LAST_UPDATED_DATE}]

keywords:  opentelemetry, otel, agent, backend, Rust, SDK, package, installation

---

{{site.data.keyword.attribute-definition-list}}

# OpenTelemetry Rust client SDK package

Instana OpenTelemetry Rust SDK is based on the open source [OpenTelemetry Rust](https://github.com/open-telemetry/opentelemetry-rust). It provides an OpenTelemetry implementation that supports Instana OpenTelemetry and IBM platforms (S390X Linux, PowerPC Linux, and AIX), as well as other platforms such as Linux x64 or ARM64 and Windows.

The SDK includes standard OpenTelemetry features that provide tools, APIs, and SDKs that are used to instrument, generate, collect, and export telemetry data (metrics, logs, and traces). These capabilities help you to analyze and understand your software's performance and behavior.

In addition to the standard features of OpenTelemetry Rust, the Instana OpenTelemetry Rust also provides Instana OpenTelemetry Rust SDK, which supports exporter, propagation, and serialization. It allows you to send OpenTelemetry trace data to Instana for monitoring and observability. For more information, see the [Instana OpenTelemetry Rust SDK](https://github.com/instana/instana-opentelemetry-rust/blob/main/opentelemetry-instana/README.md).
{: shortdesc}

## Getting started

### Download and build

The Instana OpenTelemetry Rust SDK is available in source as `tar.gz` or `zip` file and can be downloaded from the [releases](https://github.com/instana/instana-opentelemetry-rust/releases).

Before building from source, make sure that the following tools are installed:
 - Rust 1.86 or later: Only Rust 1.86 is tested on AIX.
 - gcc/g++ 11.0.0 or later (Linux and AIX): Set the environment variable `CC` to `gcc` and `CXX` to `g++`.
 - OpenSSL development package (Linux and AIX): Set the environment variable `OPENSSL_DIR` and `OPENSSL_LIB_DIR` to point to the OpenSSL installation and library directories.

To build the SDK, run the following code:

```
cargo build --release
```
{: codeblock}

### Using the crates

You can use the crates in either of the following ways:

1. Use the crate path in your `Cargo.toml` file:

    for example:

    ```
    [dependencies]
    <crate-name> = { path = "<path-to-the-root-of-source>" } 
    ```
    {: codeblock}

2. Install the crates locally by using the `cargo install` command:

    ```
    cargo install --path <path-to-the-root-of-source>
    ```
    {: codeblock}

### Examples in opentelemetry-instana

The examples in the `opentelemetry-instana` folder show how to use the Instana OpenTelemetry exporter to send trace data to Instana. For more information, see the [examples/README.md](https://github.com/instana/instana-opentelemetry-rust/blob/main/opentelemetry-instana/docs/examples.md).

### Examples for using IDOT (Instana Distribution of OpenTelemetry Collector)

Examples are also provided for sending trace data through IDOT by using the OpenTelemetry Rust SDK. For more information, see [basic-otlp on IDOT](https://github.com/instana/instana-opentelemetry-rust/blob/main/opentelemetry-otlp/examples/basic-otlp/README.md) and [basic-otlp-http on IDOT](https://github.com/instana/instana-opentelemetry-rust/blob/main/opentelemetry-otlp/examples/basic-otlp-http/README.md).
