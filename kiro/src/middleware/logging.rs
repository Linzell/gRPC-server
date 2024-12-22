// utils/logging.rs
//
// Copyright Charlie Cohen <linzellart@gmail.com>
//
// Licensed under the GNU General Public License, Version 3.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     https://www.gnu.org/licenses/gpl-3.0.html
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use std::{fmt::Debug, time::Duration};

use http::{HeaderMap, Request as HttpRequest, Response as HttpResponse};
use tokio::task;
use tower_http::{
    classify::{GrpcErrorsAsFailures, GrpcFailureClass, SharedClassifier},
    trace::{
        DefaultOnBodyChunk, DefaultOnEos, MakeSpan, OnFailure, OnRequest, OnResponse, TraceLayer,
    },
};
use tracing::{debug, error, Span};

use crate::config::{ErrorContext, ErrorSeverity, LoggingConfig};
#[cfg(feature = "mailer")]
use crate::utils::error_mailer::ErrorMailer;

const AUTH_HEADER: &str = "authorization";

/// A type alias for the complete tracing layer configuration
pub type GrpcTraceLayer = TraceLayer<
    SharedClassifier<GrpcErrorsAsFailures>,
    GrpcMakeSpan,
    LogOnRequest,
    LogOnResponse,
    DefaultOnBodyChunk,
    DefaultOnEos,
    LogOnFailure,
>;

/// Creates a new trace layer with default configuration
pub fn trace_layer(config: &crate::config::Config) -> GrpcTraceLayer {
    let classifier = SharedClassifier::new(GrpcErrorsAsFailures::default());

    TraceLayer::new(classifier)
        .make_span_with(GrpcMakeSpan::new())
        .on_request(LogOnRequest::new(config.clone()))
        .on_response(LogOnResponse::new())
        .on_failure(LogOnFailure::new(config.clone()))
}

/// # GrpcMakeSpan
///
/// A struct that generates spans for gRPC requests.
///
/// ## Example
/// ```rust
/// let span_maker = GrpcMakeSpan::new(config);
/// let span = span_maker.make_span(&request);
/// ```
#[derive(Clone, Debug)]
pub struct GrpcMakeSpan {
    logging_config: LoggingConfig,
}

impl GrpcMakeSpan {
    /// Creates a new GrpcMakeSpan instance with default config
    pub fn new() -> Self {
        Self {
            logging_config: LoggingConfig::default(),
        }
    }

    /// Determines if a request path should be logged
    fn should_log_request(&self, path: &str) -> bool {
        !self
            .logging_config
            .excluded_paths
            .contains(&path.to_string())
    }

    /// Extracts service and method names from a gRPC path
    pub fn extract_service_info(&self, path: &str) -> (String, String) {
        let path = path.trim_start_matches('/');
        let parts: Vec<&str> = path.split('/').collect();

        match parts.as_slice() {
            [service_path, method_name] => {
                let service_parts: Vec<&str> = service_path.split('.').collect();
                let service_name = service_parts
                    .iter()
                    .find(|&s| s.ends_with("Service"))
                    .and_then(|s| s.strip_suffix("Service"))
                    .unwrap_or_else(|| service_parts.last().unwrap_or(&"unknown"))
                    .to_string();
                (service_name, method_name.to_string())
            }
            // Handle paths like "v1/service.name/method"
            [version, service_path, method_name] if *version == "v1" => {
                let service_parts: Vec<&str> = service_path.split('.').collect();
                let service_name = service_parts
                    .last()
                    .and_then(|s| s.strip_suffix("Service"))
                    .unwrap_or_else(|| service_parts.last().unwrap_or(&"unknown"))
                    .to_string();
                (service_name, method_name.to_string())
            }
            _ => ("unknown".to_string(), "unknown".to_string()),
        }
    }
}

impl<B> MakeSpan<B> for GrpcMakeSpan {
    fn make_span(&mut self, request: &HttpRequest<B>) -> Span {
        let path = request.uri().path();

        if !self.should_log_request(path) {
            return tracing::info_span!("health-check");
        }

        let (service_name, method_name) = self.extract_service_info(path);
        let has_auth = request.headers().contains_key(AUTH_HEADER);

        tracing::info_span!(
            "grpc-request",
            %path,
            %service_name,
            %method_name,
            authenticated = %has_auth,
            target = %format!("{}/{}", service_name, method_name)
        )
    }
}

/// Handles logging of incoming requests
#[derive(Debug, Clone)]
pub struct LogOnRequest {
    config: crate::config::Config,
    logging_config: LoggingConfig,
}

impl LogOnRequest {
    pub fn new(config: crate::config::Config) -> Self {
        Self {
            config,
            logging_config: LoggingConfig::default(),
        }
    }

    fn log_auth_status(&self, headers: &HeaderMap, span: &Span, path: &str) {
        if self
            .logging_config
            .public_endpoints
            .contains(&path.to_string())
        {
            return;
        }

        match headers.get(AUTH_HEADER) {
            Some(auth) => {
                if let Ok(auth_str) = auth.to_str() {
                    if auth_str.starts_with("Bearer ") {
                        debug!(
                            parent: span,
                            "Request authenticated with Bearer token"
                        );
                        return;
                    }
                }
                debug!(
                    parent: span,
                    "Invalid authentication format"
                );
            }
            None => debug!(
                parent: span,
                "Request missing authentication"
            ),
        }
    }
}

impl<B> OnRequest<B> for LogOnRequest {
    fn on_request(&mut self, request: &HttpRequest<B>, span: &Span) {
        let path = request.uri().path();

        if self
            .logging_config
            .excluded_paths
            .contains(&path.to_string())
        {
            return;
        }

        let (service_name, method_name) = GrpcMakeSpan::new().extract_service_info(path);

        self.log_auth_status(request.headers(), span, path);

        debug!(
            target: "grpc-request",
            path = %path,
            service_name = %service_name,
            method_name = %method_name,
            headers = ?request.headers(),
            environment = ?self.config.app.environment,
            "Incoming request to {}/{}",
            service_name,
            method_name
        );
    }
}

/// Handles logging of responses
#[derive(Debug, Clone)]
pub struct LogOnResponse {}

impl LogOnResponse {
    pub fn new() -> Self {
        Self {}
    }
}

impl<B> OnResponse<B> for LogOnResponse {
    fn on_response(self, response: &HttpResponse<B>, latency: Duration, _span: &Span) {
        if response.status().is_success() {
            return;
        }

        debug!(
            target: "grpc-response",
            "{:?} {} {:?} {:?}",
            response.version(),
            response.status(),
            response.headers(),
            latency
        );
    }
}

/// Handles logging of failures
#[derive(Debug, Clone)]
pub struct LogOnFailure {
    #[cfg(feature = "mailer")]
    config: crate::config::Config,
    logging_config: LoggingConfig,
}

impl LogOnFailure {
    pub fn new(
        #[cfg(feature = "mailer")] config: crate::config::Config,
        #[cfg(not(feature = "mailer"))] _config: crate::config::Config,
    ) -> Self {
        Self {
            #[cfg(feature = "mailer")]
            config,
            logging_config: LoggingConfig::default(),
        }
    }

    fn determine_severity(&self, failure: &GrpcFailureClass) -> ErrorSeverity {
        match failure {
            GrpcFailureClass::Code(code) => match code.get() {
                16 => ErrorSeverity::Low,      // Authentication failures are expected
                13 => ErrorSeverity::Critical, // Internal errors are critical
                14 => ErrorSeverity::Critical, // Unavailable service is critical
                15 => ErrorSeverity::Critical, // Data loss is critical
                12 => ErrorSeverity::Low,      // Unimplemented is low severity
                _ => ErrorSeverity::Medium,
            },
            GrpcFailureClass::Error(_) => ErrorSeverity::High,
        }
    }

    fn determine_error_cause(&self, failure: &GrpcFailureClass) -> String {
        match failure {
            GrpcFailureClass::Code(code) => match code.get() {
                1 => "Operation cancelled".to_string(),
                2 => "Unknown error".to_string(),
                3 => "Invalid argument".to_string(),
                4 => "Deadline exceeded".to_string(),
                5 => "Not found".to_string(),
                6 => "Already exists".to_string(),
                7 => "Permission denied".to_string(),
                8 => "Resource exhausted".to_string(),
                9 => "Failed precondition".to_string(),
                10 => "Operation aborted".to_string(),
                11 => "Out of range".to_string(),
                12 => "Operation not implemented".to_string(),
                13 => "Internal server error".to_string(),
                14 => "Service unavailable".to_string(),
                15 => "Data loss".to_string(),
                16 => "Authentication failed".to_string(),
                _ => "Unknown error".to_string(),
            },
            GrpcFailureClass::Error(error) => error.to_string(),
        }
    }

    async fn handle_error(&self, context: ErrorContext) {
        error!(
            target: "grpc-failure",
            "{} ERROR ThreadId({:02}) grpc-request{{path={} service_name={} method_name={} authenticated=false target={}/{}}}: kiro::utils::{}: {}",
            chrono::Utc::now().format(&self.logging_config.format),
            std::process::id() % 100,
            context.service,
            context.endpoint,
            context.message,
            context.service,
            context.endpoint,
            context.error_source,
            context.clone().cause.unwrap_or_else(|| "Unknown error".to_string())
        );

        #[cfg(feature = "mailer")]
        if !context.error_source.contains("auth")
            && context.severity >= self.logging_config.email_severity_threshold
        {
            let mailer = ErrorMailer::new(self.config.clone());
            if let Err(e) = mailer.send_error_notification(&context).await {
                error!(
                    error = %e,
                    request_id = %context.request_id,
                    "Failed to send error notification email"
                );
            }
        }
    }
}

impl OnFailure<GrpcFailureClass> for LogOnFailure {
    fn on_failure(
        &mut self, failure_classification: GrpcFailureClass, latency: Duration, span: &Span,
    ) {
        let path = span
            .metadata()
            .and_then(|m| m.fields().field("path"))
            .map(|f| f.to_string())
            .unwrap_or_else(|| "unknown".to_string());

        let (service_name, method_name) = GrpcMakeSpan::new().extract_service_info(&path);

        let location = span
            .metadata()
            .map(|m| m.target())
            .unwrap_or("unknown")
            .replace("kiro::utils::logging", "auth");

        let error_source = format!(
            "Service: {}, Method: {}, Error Code: {}, Location: {}",
            service_name, method_name, failure_classification, location
        );

        let request_details = format!(
            "Service: {}, Method: {}, Path: {}, Duration: {:?}",
            service_name, method_name, path, latency
        );

        let context = ErrorContext::builder(self.determine_severity(&failure_classification))
            .error_code(failure_classification.to_string())
            .message(format!("GRPC request failed: {}", failure_classification))
            .service(service_name)
            .endpoint(method_name)
            .error_source(error_source)
            .request_details(request_details)
            .cause(Some(self.determine_error_cause(&failure_classification)))
            .stack_trace(Some(format!("Latency: {:?}\nSpan: {:?}", latency, span)))
            .build();

        let this = self.clone();
        task::spawn(async move {
            this.handle_error(context).await;
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::Config;
    use http::HeaderValue;

    fn create_test_config() -> Config {
        Config::init().unwrap()
    }

    #[test]
    fn test_grpc_make_span_new() {
        let span_maker = GrpcMakeSpan::new();
        assert!(!span_maker.logging_config.excluded_paths.is_empty());
    }

    #[test]
    fn test_extract_service_info_standard_path() {
        let span_maker = GrpcMakeSpan::new();
        let (service, method) =
            span_maker.extract_service_info("payment.v1.StripeService/ReadSubscriptions");
        assert_eq!(service, "Stripe");
        assert_eq!(method, "ReadSubscriptions");
    }

    #[test]
    fn test_extract_service_info_v1_path() {
        let span_maker = GrpcMakeSpan::new();
        let (service, method) = span_maker.extract_service_info("v1/service.name/method");
        assert_eq!(service, "name");
        assert_eq!(method, "method");
    }

    #[test]
    fn test_log_auth_status() {
        let config = create_test_config();
        let logger = LogOnRequest::new(config);
        let mut headers = HeaderMap::new();
        headers.insert(AUTH_HEADER, HeaderValue::from_static("Bearer token123"));
        let span = tracing::info_span!("test");
        logger.log_auth_status(&headers, &span, "/test/path");
    }
}
