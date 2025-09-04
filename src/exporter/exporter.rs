use crate::exporter::defs;
use crate::exporter::serialize_span;
use opentelemetry_sdk::error::{OTelSdkError, OTelSdkResult};

use opentelemetry_sdk::Resource;
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc, Mutex,
};

use http::{header::CONTENT_TYPE, Method};
use opentelemetry::Value;
use opentelemetry_http::HttpClient;
use opentelemetry_sdk::trace::SpanExporter;
use std::env;
use thiserror::Error;
use url::Url;

#[derive(Debug, PartialEq, Clone)]
pub struct Options {
    pub endpoint: String,
    pub hostname: String,
    pub source_address: String,
    pub service: String,
    pub headers: http::HeaderMap,
}

impl Default for Options {
    fn default() -> Self {
        let mut headers_ = http::HeaderMap::new();
        headers_.insert(
            CONTENT_TYPE,
            http::HeaderValue::from_static("application/json"),
        );

        let host = env::var("INSTANA_AGENT_HOST")
            .unwrap_or_else(|_| defs::DEFAULT_INSTANA_AGENT_HOST.to_string());

        let port = env::var("INSTANA_AGENT_PORT")
            .unwrap_or_else(|_| defs::DEFAULT_INSTANA_AGENT_PORT.to_string());

        Options {
            endpoint: format!(
                "http://{}:{}/com.instana.plugin.generic.rawtrace",
                host, port
            ),
            hostname: String::new(),
            source_address: String::new(),
            service: String::new(),
            headers: headers_,
        }
    }
}

impl Options {
    pub fn with_endpoint(endpoint: &str) -> Option<Self> {
        match Url::parse(endpoint) {
            Ok(url) => Some(Self {
                endpoint: url.to_string(),
                ..Default::default()
            }),
            Err(_) => None,
        }
    }
}

#[derive(Error, Debug)]
/// Errors that can occur while building an exporter.
#[non_exhaustive]
pub enum BuildError {
    /// Spawning a new thread failed.
    #[error("Spawning a new thread failed. Unable to create Reqwest-Blocking client.")]
    ThreadSpawnFailed,

    /// No Http client specified.
    #[error("no http client specified")]
    NoHttpClient,
}

#[derive(Debug)]
pub struct Exporter {
    client_: Mutex<Option<Arc<dyn HttpClient>>>,
    options_: Options,
    is_shutdown_: AtomicBool,
    resource_: opentelemetry_sdk::Resource,
}

impl PartialEq for Exporter {
    fn eq(&self, other: &Self) -> bool {
        self.options_ == other.options_
            && self.is_shutdown_.load(Ordering::SeqCst) == other.is_shutdown_.load(Ordering::SeqCst)
            && self.resource_ == other.resource_
    }
}

impl Default for Exporter {
    fn default() -> Self {
        let options = Options::default();
        let is_shutdown = AtomicBool::new(false);

        let resource = Resource::builder_empty().build();
        Self {
            options_: options,
            is_shutdown_: is_shutdown,
            client_: Mutex::new(None),
            resource_: resource,
        }
    }
}

impl SpanExporter for Exporter {
    async fn export(&self, batch: Vec<opentelemetry_sdk::trace::SpanData>) -> OTelSdkResult {
        // Get resource
        self.get_resource();

        // Check if exporter is shutdown
        if self.is_shutdown_.load(Ordering::SeqCst) {
            return Err(OTelSdkError::AlreadyShutdown);
        }

        // Get client
        let client = match self
            .client_
            .lock()
            .map_err(|e| OTelSdkError::InternalFailure(format!("Mutex lock failed: {}", e)))
            .and_then(|g| match &*g {
                Some(client) => Ok(Arc::clone(client)),
                _ => Err(OTelSdkError::AlreadyShutdown),
            }) {
            Ok(client) => client,
            Err(err) => return Err(err),
        };

        // Serialize batch to JSON bytes
        let export_body = match serialize_span::serialize_batch(self, &batch) {
            Ok(body) => body,
            Err(e) => {
                return Err(OTelSdkError::InternalFailure(format!(
                    "Serialization error: {}",
                    e
                )))
            },
        };

        // Build request
        let mut request = match http::Request::builder()
            .method(Method::POST)
            .uri(&self.options_.endpoint)
            .header(CONTENT_TYPE, "application/json")
            .body(export_body)
        {
            Ok(req) => req,
            Err(e) => return Err(OTelSdkError::InternalFailure(e.to_string())),
        };

        // Add headers
        for (k, v) in &self.options_.headers {
            request.headers_mut().insert(k.clone(), v.clone());
        }

        // Send request
        let response = client
            .send_bytes(request)
            .await
            .map_err(|e| OTelSdkError::InternalFailure(format!("{e:?}")))?;

        // Check response
        if !response.status().is_success() {
            let error = format!(
                "OpenTelemetry trace export failed. Url: {}, Status Code: {}, Response: {:?}",
                self.options_.endpoint,
                response.status().as_u16(),
                response.body()
            );
            return Err(OTelSdkError::InternalFailure(error));
        }

        Ok(())
    }

    fn shutdown(&mut self) -> opentelemetry_sdk::error::OTelSdkResult {
        self.is_shutdown_.store(true, Ordering::SeqCst);
        let mut client_guard = self.client_.lock().map_err(|e| {
            OTelSdkError::InternalFailure(format!("Failed to acquire client lock: {}", e))
        })?;

        if client_guard.take().is_none() {
            return Err(OTelSdkError::AlreadyShutdown);
        }
        Ok(())
    }

    fn set_resource(&mut self, _resource: &opentelemetry_sdk::Resource) {
        self.resource_ = _resource.clone();
    }

    fn force_flush(&mut self) -> opentelemetry_sdk::error::OTelSdkResult {
        Ok(())
    }
}

#[derive(Default)]
pub struct Builder {
    exporter: Exporter,
}

impl Builder {
    pub fn with_service(mut self, service: Resource) -> Self {
        self.exporter.resource_ = service;
        return self;
    }

    pub fn with_options(mut self, options: Options) -> Self {
        self.exporter.options_ = options;
        return self;
    }

    pub fn with_http_client(mut self, client: impl HttpClient + 'static) -> Self {
        self.exporter.client_ = Mutex::new(Some(Arc::new(client)));
        return self;
    }

    pub fn build(self) -> Result<Exporter, BuildError> {
        let mut http_client = self.exporter.client_.lock().unwrap().take();
        if http_client.is_none() {
            let client_result = std::thread::spawn(move || {
                reqwest::blocking::Client::builder()
                    .build()
                    .unwrap_or_else(|_| reqwest::blocking::Client::new())
            })
            .join();
            match client_result {
                Ok(client) => {
                    http_client = Some(Arc::new(client) as Arc<dyn HttpClient>);
                },
                Err(_) => {
                    return Err(BuildError::ThreadSpawnFailed);
                },
            }
        }
        let http_client = http_client.ok_or(BuildError::NoHttpClient)?;
        Ok(Exporter::new(
            http_client,
            self.exporter.options_,
            self.exporter.resource_,
        ))
    }
}

impl Exporter {
    pub fn builder() -> Builder {
        Builder {
            exporter: Exporter::default(),
        }
    }

    pub fn get_resource(&self) -> Resource {
        self.resource_.clone()
    }

    pub fn get_service_name(&self) -> Option<Value> {
        self.resource_.get(&"service.name".into())
    }

    pub fn get_resource_attributes(&self) -> Resource {
        self.resource_.clone()
    }

    pub fn get_process_pid(&self) -> Option<Value> {
        self.resource_.get(&"process.pid".into())
    }

    pub fn get_host_id(&self) -> Option<Value> {
        self.resource_.get(&"host.id".into())
    }

    pub fn get_cloud_provider(&self) -> Option<Value> {
        self.resource_.get(&"cloud.provider".into())
    }

    pub fn get_options(&self) -> Options {
        self.options_.clone()
    }

    pub fn new(client: Arc<dyn HttpClient>, options: Options, resource: Resource) -> Self {
        Self {
            options_: options,
            is_shutdown_: AtomicBool::new(false),
            client_: Mutex::new(Some(client)),
            resource_: resource,
        }
    }

    pub fn build_client(&mut self) {}
}
