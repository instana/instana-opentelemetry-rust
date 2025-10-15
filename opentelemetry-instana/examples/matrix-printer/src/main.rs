use actix_web::{web, App, HttpResponse, HttpServer, Responder};
use opentelemetry::propagation::TextMapPropagator;
use std::collections::VecDeque;
use std::sync::Arc;
use std::sync::Mutex;
use opentelemetry_instana::{InstanaExporter,InstanaPropagator};
use opentelemetry::{global, KeyValue};
use opentelemetry_sdk::Resource;
use std::sync::OnceLock;
use actix_cors::Cors;
use actix_web::HttpRequest;
use opentelemetry::trace::{Span, Tracer};
use std::collections::HashMap;
use std::fs;
use std::path::Path;

struct AppState {
    matrix_results: Mutex<VecDeque<String>>, // Store recent results
}

// Serve the HTML page
async fn matrix_page(_req: HttpRequest) -> impl Responder {
    let html_path = Path::new("src/index.html");
    let html_content = fs::read_to_string(html_path)
        .unwrap_or_else(|_| "<h1>Error: Could not load index.html</h1>".to_string());

    HttpResponse::Ok()
        .content_type("text/html")
        .body(html_content)
}
// Receive matrix result and store it
async fn matrix_result_api(
    req: HttpRequest,
    matrix_data: String,
    state: web::Data<Arc<AppState>>,
) -> impl Responder {
    let propagator = InstanaPropagator::new();

    let mut extractor: HashMap<String, String> = HashMap::new();
    for (key, value) in req.headers().iter() {
        if let Ok(val_str) = value.to_str() {
            extractor.insert(key.to_string(), val_str.to_string());
        }
    }

    let _headers = req.headers();
    let parent_context = propagator.extract(&extractor);

    let tracer = global::tracer("matrix-multiplication");
    let mut span = tracer
            .span_builder("span_with_propgated_parent")
            .with_kind(opentelemetry::trace::SpanKind::Server)
            .start_with_context(&tracer, &parent_context);

    span.set_attribute(KeyValue::new("http.method", "GET"));
    span.set_attribute(KeyValue::new(
        "http.url",
        "http://127.0.0.1:8083/matrix_result",
    ));
    span.set_attribute(KeyValue::new("http.status", 200));
    span.end();

    let mut results = state.matrix_results.lock().unwrap();
    results.push_back(matrix_data.clone());

    if results.len() > 5 {
        // Store last 5 results
        results.pop_front();
    }

    HttpResponse::Ok().body("Matrix received")
}

// Provide the latest matrix result to the UI
async fn latest_matrix(state: web::Data<Arc<AppState>>) -> impl Responder {
    let results = state.matrix_results.lock().unwrap();
    let latest_result = results
        .back()
        .cloned()
        .unwrap_or("No result yet".to_string());
    HttpResponse::Ok().body(latest_result)
}

fn get_resource() -> Resource {
    static RESOURCE: OnceLock<Resource> = OnceLock::new();
    RESOURCE
        .get_or_init(|| {
            Resource::builder()
                .with_service_name("matrix_printer_using_propgated_context")
                .with_attribute(KeyValue::new("process.pid", std::process::id() as i64))
                .build()
        })
        .clone()
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Initialize app state
    let app_state = Arc::new(AppState {
        matrix_results: Mutex::new(VecDeque::new()),
    });

    let instana_exporter = InstanaExporter::builder()
        .with_service(get_resource())
        .build()
        .expect("Failed to create instana exporter");

    let tracer_provider = opentelemetry_sdk::trace::SdkTracerProvider::builder()
        .with_batch_exporter(instana_exporter)
        .with_resource(get_resource())
        .build();
    global::set_tracer_provider(tracer_provider.clone());

    println!("Starting matrix-printer server on http://127.0.0.1:8083");

    // Create and run the server
    HttpServer::new(move || {
        App::new()
            .wrap(Cors::permissive())
            .app_data(web::Data::new(app_state.clone()))
            .route("/", web::get().to(matrix_page))
            .route("/matrix_result", web::post().to(matrix_result_api))
            .route("/latest_matrix", web::get().to(latest_matrix))
    })
    .bind("127.0.0.1:8083")?
    .run()
    .await
}
