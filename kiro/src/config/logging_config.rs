// utils/logging_config.rs
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

/// Error severity levels for logging and notifications
#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub enum ErrorSeverity {
    /// Critical errors that require immediate attention
    Critical,
    /// High priority errors that should be addressed soon
    High,
    /// Medium priority issues that should be investigated
    Medium,
    /// Low priority issues that can be addressed later
    Low,
}

impl std::fmt::Display for ErrorSeverity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ErrorSeverity::Critical => write!(f, "Critical"),
            ErrorSeverity::High => write!(f, "High"),
            ErrorSeverity::Medium => write!(f, "Medium"),
            ErrorSeverity::Low => write!(f, "Low"),
        }
    }
}

/// Context information for error logging and reporting
#[derive(Debug, Clone)]
pub struct ErrorContext {
    /// Severity level of the error
    #[cfg(feature = "mailer")]
    pub severity: ErrorSeverity,
    /// Error code identifier
    #[cfg(feature = "mailer")]
    pub error_code: String,
    /// Human readable error message
    pub message: String,
    /// Service where the error occurred
    pub service: String,
    /// Endpoint where the error occurred
    pub endpoint: String,
    /// Unique request identifier
    #[cfg(feature = "mailer")]
    pub request_id: String,
    /// Optional stack trace information
    #[cfg(feature = "mailer")]
    pub stack_trace: Option<String>,
    /// Optional error cause
    pub cause: Option<String>,
    /// Source of the error
    pub error_source: String,
    /// Details about the request that caused the error
    #[cfg(feature = "mailer")]
    pub request_details: String,
}

/// Builder for ErrorContext
pub struct ErrorContextBuilder {
    #[cfg(feature = "mailer")]
    severity: ErrorSeverity,
    error_code: String,
    message: String,
    service: String,
    endpoint: String,
    error_source: String,
    request_details: String,
    cause: Option<String>,
    stack_trace: Option<String>,
}

impl ErrorContextBuilder {
    pub fn new(
        #[cfg(feature = "mailer")] severity: ErrorSeverity,
        #[cfg(not(feature = "mailer"))] _severity: ErrorSeverity,
    ) -> Self {
        Self {
            #[cfg(feature = "mailer")]
            severity,
            error_code: String::new(),
            message: String::new(),
            service: String::new(),
            endpoint: String::new(),
            error_source: String::new(),
            request_details: String::new(),
            cause: None,
            stack_trace: None,
        }
    }

    pub fn error_code(mut self, code: impl Into<String>) -> Self {
        self.error_code = code.into();
        self
    }

    pub fn message(mut self, msg: impl Into<String>) -> Self {
        self.message = msg.into();
        self
    }

    pub fn service(mut self, service: impl Into<String>) -> Self {
        self.service = service.into();
        self
    }

    pub fn endpoint(mut self, endpoint: impl Into<String>) -> Self {
        self.endpoint = endpoint.into();
        self
    }

    pub fn error_source(mut self, source: impl Into<String>) -> Self {
        self.error_source = source.into();
        self
    }

    pub fn request_details(mut self, details: impl Into<String>) -> Self {
        self.request_details = details.into();
        self
    }

    pub fn cause(mut self, cause: Option<String>) -> Self {
        self.cause = cause;
        self
    }

    pub fn stack_trace(mut self, trace: Option<String>) -> Self {
        self.stack_trace = trace;
        self
    }

    pub fn build(self) -> ErrorContext {
        ErrorContext {
            #[cfg(feature = "mailer")]
            severity: self.severity,
            #[cfg(feature = "mailer")]
            error_code: self.error_code,
            message: self.message,
            service: self.service,
            endpoint: self.endpoint,
            #[cfg(feature = "mailer")]
            request_id: uuid::Uuid::new_v4().to_string(),
            #[cfg(feature = "mailer")]
            stack_trace: self.stack_trace,
            cause: self.cause,
            error_source: self.error_source,
            #[cfg(feature = "mailer")]
            request_details: self.request_details,
        }
    }
}

impl ErrorContext {
    pub fn builder(severity: ErrorSeverity) -> ErrorContextBuilder {
        ErrorContextBuilder::new(severity)
    }
}

/// Configuration for logging behavior
#[derive(Debug, Clone)]
pub struct LoggingConfig {
    /// Log message format string
    pub format: String,
    /// Paths to exclude from logging
    pub excluded_paths: Vec<String>,
    /// Public endpoints that don't require authentication
    pub public_endpoints: Vec<String>,
    /// Admin-only endpoints
    pub admin_endpoints: Vec<String>,
    /// Minimum severity threshold for email notifications
    #[cfg(feature = "mailer")]
    pub email_severity_threshold: ErrorSeverity,
}

impl Default for LoggingConfig {
    fn default() -> Self {
        Self {
            format: "%Y-%m-%dT%H:%M:%S%.3fZ %L %t %m".to_string(),
            excluded_paths: vec![
                "/grpc.health.v1.Health/Check".to_string(),
                "/grpc.reflection.v1alpha.ServerReflection/ServerReflectionInfo".to_string(),
            ],
            public_endpoints: vec![
                // Auth Service public endpoints
                "/v1.AuthService/Login".to_string(),
                "/v1.AuthService/Register".to_string(),
                // Payment Service public endpoints
                "/v1.PaymentService/WebhookHandler".to_string(),
            ],
            admin_endpoints: vec![
                // Admin Service endpoints
                "/v1.AdminService/ReadOrganizationsAdmin".to_string(),
                "/v1.AdminService/ReadUsersAdmin".to_string(),
                "/v1.AdminService/RegenerateInvitationCode".to_string(),
                "/v1.AdminService/SendResetEmail".to_string(),
                "/v1.AdminService/SendResetPassword".to_string(),
                "/v1.AdminService/UpdateActivation".to_string(),
                "/v1.AdminService/UpdateIsAdmin".to_string(),
            ],
            #[cfg(feature = "mailer")]
            email_severity_threshold: ErrorSeverity::High,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_severity_display() {
        assert_eq!(ErrorSeverity::Critical.to_string(), "Critical");
        assert_eq!(ErrorSeverity::High.to_string(), "High");
        assert_eq!(ErrorSeverity::Medium.to_string(), "Medium");
        assert_eq!(ErrorSeverity::Low.to_string(), "Low");
    }

    #[test]
    fn test_error_context_creation() {
        let context = ErrorContext::builder(ErrorSeverity::High)
            .error_code("ERR001")
            .message("Test error")
            .service("test_service")
            .endpoint("test_endpoint")
            .error_source("test_source")
            .request_details("test_details")
            .cause(Some("test cause".to_string()))
            .stack_trace(Some("test stack trace".to_string()))
            .build();

        #[cfg(feature = "mailer")]
        assert_eq!(context.severity, ErrorSeverity::High);
        #[cfg(feature = "mailer")]
        assert_eq!(context.error_code, "ERR001");
        assert_eq!(context.message, "Test error");
        assert_eq!(context.service, "test_service");
        assert_eq!(context.endpoint, "test_endpoint");
        #[cfg(feature = "mailer")]
        assert!(uuid::Uuid::parse_str(&context.request_id).is_ok());
    }

    #[test]
    fn test_logging_config_default() {
        let config = LoggingConfig::default();
        assert!(config
            .excluded_paths
            .contains(&"/grpc.health.v1.Health/Check".to_string()));
        assert!(config
            .admin_endpoints
            .contains(&"/v1.AdminService/ReadUsersAdmin".to_string()));
        #[cfg(feature = "mailer")]
        assert_eq!(config.email_severity_threshold, ErrorSeverity::High);
    }
}
