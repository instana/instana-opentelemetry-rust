use actix_cors::Cors;
use actix_web::{web, App, HttpResponse, HttpServer, Responder};
use opentelemetry_instana::{InstanaExporter,InstanaPropagator};
use opentelemetry::global;
use opentelemetry::propagation::TextMapPropagator;
use opentelemetry::trace::{Span,Tracer};
use opentelemetry::{Context, KeyValue};
use opentelemetry_sdk::Resource;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::sync::OnceLock;
use opentelemetry::trace::TraceContextExt;
use reqwest::header::{HeaderMap, HeaderName, HeaderValue};
use std::collections::HashMap;
use std::fs;
use std::path::Path;

#[derive(Debug, Deserialize)]
struct MatrixInput {
    a: [[i32; 3]; 3],
    b: [[i32; 3]; 3],
}

#[derive(Serialize)]
struct MatrixResult {
    result: [[i32; 3]; 3],
}

// Function to multiply matrices
fn multiply_matrices(a: [[i32; 3]; 3], b: [[i32; 3]; 3]) -> [[i32; 3]; 3] {
    let mut result = [[0; 3]; 3];
    for i in 0..3 {
        for j in 0..3 {
            for k in 0..3 {
                result[i][j] += a[i][k] * b[k][j];
            }
        }
    }
    result
}

// API Endpoint: Receives matrices, returns result
async fn multiply_matrices_api(matrix_data: web::Json<MatrixInput>) -> impl Responder {
    let result = multiply_matrices(matrix_data.a, matrix_data.b);
    let client = Client::new();

    let formatted_matrix = result
        .iter()
        .map(|row| {
            row.iter()
                .map(ToString::to_string)
                .collect::<Vec<_>>()
                .join(" ")
        })
        .collect::<Vec<_>>()
        .join("\n");

    let tracer = global::tracer("http_client_tracer");
    let mut span = tracer.start("outgoing_http_request_tracestate");
    span.set_attribute(KeyValue::new("http.method", "POST"));
    span.set_attribute(KeyValue::new(
        "http.url",
        "http://127.0.0.1:8083/matrix_result",
    ));
    span.set_attribute(KeyValue::new("http.status", 200));

    let propagator = InstanaPropagator::new();
    let mut injector: HashMap<String, String> = HashMap::new();
    let mut headers = HeaderMap::new();

    let cx = Context::current_with_span(span);
    propagator.inject_context(&cx, &mut injector);
    for (key, value) in &injector {
        if let (Ok(name), Ok(val)) = (
            HeaderName::from_bytes(key.as_bytes()),
            HeaderValue::from_bytes(value.as_bytes()),
        ) {
            headers.insert(name, val);
        }
    }

    let _ = client
        .post("http://127.0.0.1:8083/matrix_result")
        .body(formatted_matrix)
        .header("Content-Type", "text/plain")
        .headers(headers)
        .send()
        .await;
    let span = cx.span();

    span.end();

    HttpResponse::Ok().json(MatrixResult { result })
}

fn get_resource() -> Resource {
    static RESOURCE: OnceLock<Resource> = OnceLock::new();
    RESOURCE
        .get_or_init(|| {
            Resource::builder()
                .with_service_name("matrix-multplier_propgation")
                .with_attribute(KeyValue::new("process.pid", std::process::id() as i64))
                .build()
        })
        .clone()
}

// Serve the HTML page
async fn index() -> impl Responder {
    let html_path = Path::new("src/index.html");
    let html_content = fs::read_to_string(html_path)
        .unwrap_or_else(|_| "<h1>Error: Could not load index.html</h1>".to_string());

    HttpResponse::Ok()
        .content_type("text/html")
        .body(html_content)
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let instana_exporter = InstanaExporter::builder()
        .with_service(get_resource())
        .build()
        .expect("Failed to create instana exporter");

    let tracer_provider = opentelemetry_sdk::trace::SdkTracerProvider::builder()
        .with_batch_exporter(instana_exporter)
        .with_resource(get_resource())
        .build();

    global::set_tracer_provider(tracer_provider.clone());
    let _tracer = global::tracer("matrix_multiplier");

    println!("Starting matrix multiplication server on http://127.0.0.1:8081");
    HttpServer::new(|| {
        App::new()
            .wrap(Cors::permissive())
            .route("/", web::get().to(index))  // Serve HTML page
            .route("/multiply", web::post().to(multiply_matrices_api)) // API Endpoint
    })
    .bind("127.0.0.1:8081")?
    .run()
    .await
}
