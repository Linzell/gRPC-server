// utils/error_mailer.rs
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

use chrono::Utc;
use kiro_database::get_env_or;
use kiro_mailer::{ContentType, Mailer, MailerTrait};
use tonic::Status;

use crate::config::{Environment, ErrorContext, ErrorSeverity};

/// Error mailer for sending notifications about system errors
///
/// Sends formatted error emails to configured recipients based on environment and error severity
#[derive(Debug)]
pub struct ErrorMailer {
    config: crate::config::Config,
    smtp_user: String,
    support_email: String,
}

impl ErrorMailer {
    /// Creates a new ErrorMailer with default configuration
    pub fn new(config: crate::config::Config) -> Self {
        Self {
            config,
            smtp_user: get_env_or("SMTP_USER", "user"),
            support_email: "support@digitalkin.ai".to_string(),
        }
    }

    /// Sends an error notification email if severity warrants it
    ///
    /// # Arguments
    /// * `context` - Error context containing details about the error
    ///
    /// # Returns
    /// * `Ok(())` if email sent successfully or skipped
    /// * `Err(Status)` if email sending failed
    pub async fn send_error_notification(&self, context: &ErrorContext) -> Result<(), Status> {
        if !self.should_send_email(&context.severity) {
            return Ok(());
        }

        let template = self.build_error_template(context).await?;
        let recipient = self.get_recipient();
        let subject = self.build_subject(context);

        self.send_email(&recipient, &subject, template).await
    }

    /// Determines if an error notification email should be sent based on severity and environment
    fn should_send_email(&self, severity: &ErrorSeverity) -> bool {
        matches!(severity, ErrorSeverity::Critical | ErrorSeverity::High)
            && self.config.app.environment.is_production()
    }

    /// Builds the error email template by replacing placeholders with error context
    async fn build_error_template(&self, context: &ErrorContext) -> Result<String, Status> {
        let timestamp = Utc::now().to_rfc2822();
        let owner = get_env_or("ERROR_SERVER_OWNER", "Unknown");
        let email = get_env_or("SMTP_USER", "Unknown");

        Mailer::load_template("error.html")
            .await
            .map_err(|e| Status::internal(format!("Failed to load error template: {}", e)))
            .map(|template| {
                template
                    .replace("${{ERROR_MSG}}", &context.message)
                    .replace("${{ERROR_DATE}}", &timestamp)
                    .replace("${{ERROR_SERVICE}}", &context.service)
                    .replace(
                        "${{ERROR_ENVIRONMENT}}",
                        &format!("{:?}", self.config.app.environment),
                    )
                    .replace("${{ERROR_TYPE}}", &context.error_code)
                    .replace("${{ERROR_SEVERITY}}", &context.severity.to_string())
                    .replace(
                        "${{ERROR_CAUSE}}",
                        &context
                            .cause
                            .clone()
                            .unwrap_or_else(|| "Unknown".to_string()),
                    )
                    .replace("${{ERROR_METHOD}}", &context.endpoint)
                    .replace("${{ERROR_REQUEST_ID}}", &context.request_id)
                    .replace("${{ERROR_SOURCE}}", &context.error_source)
                    .replace("${{ERROR_DETAILS}}", &context.request_details)
                    .replace(
                        "${{STACK_TRACE}}",
                        &format!(
                            "Stack Trace:\n{}\n\nError Message:\n{}\n\nRequest Details:\n{}",
                            context.stack_trace.as_deref().unwrap_or("Not available"),
                            context.message,
                            context.request_details
                        ),
                    )
                    .replace("${{ERROR_SERVER_OWNER}}", &owner)
                    .replace("${{ERROR_SUPPORT_EMAIL}}", &email)
            })
    }

    /// Gets the appropriate email recipient based on environment
    fn get_recipient(&self) -> String {
        match self.config.app.environment {
            Environment::Production => self.support_email.clone(),
            _ => self.smtp_user.clone(),
        }
    }

    /// Builds the email subject line from the error context
    fn build_subject(&self, context: &ErrorContext) -> String {
        format!(
            "[{}] {} Alert - {} in {}/{}",
            format!("{:?}", self.config.app.environment).to_uppercase(),
            context.severity,
            context.error_code,
            context.service,
            context.endpoint
        )
    }

    /// Sends the formatted error email
    async fn send_email(
        &self, recipient: &str, subject: &str, template: String,
    ) -> Result<(), Status> {
        let origin = get_env_or("SMTP_USER", "Unknown");

        let message = Mailer::build_mail(
            &origin,
            recipient,
            subject,
            ContentType::TEXT_HTML,
            template,
        )
        .map_err(|_| Status::internal("Failed to build error notification"))?;

        Mailer::new()
            .send_mail(message)
            .await
            .map_err(|_| Status::internal("Failed to send error notification"))?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{AppConfig, Config};

    fn create_test_config(env: Environment) -> Config {
        Config {
            app: AppConfig {
                frontend_url: "test".to_string(),
                environment: env,
                enable_tracing: false,
            },
            ports: crate::config::Ports::init().unwrap(),
        }
    }

    #[test]
    fn test_should_send_email() {
        let mailer = ErrorMailer::new(create_test_config(Environment::Production));

        assert!(mailer.should_send_email(&ErrorSeverity::Critical));
        assert!(mailer.should_send_email(&ErrorSeverity::High));
        assert!(!mailer.should_send_email(&ErrorSeverity::Medium));
        assert!(!mailer.should_send_email(&ErrorSeverity::Low));

        let development_mailer = ErrorMailer::new(create_test_config(Environment::Development));
        assert!(!development_mailer.should_send_email(&ErrorSeverity::Critical));
    }

    #[test]
    fn test_get_recipient() {
        let mailer = ErrorMailer::new(create_test_config(Environment::Production));
        assert_eq!(mailer.get_recipient(), "test@email.com");

        let development_mailer = ErrorMailer::new(create_test_config(Environment::Development));
        assert_eq!(development_mailer.get_recipient(), "user");
    }

    #[test]
    fn test_build_subject() {
        let mailer = ErrorMailer::new(create_test_config(Environment::Production));

        let context = ErrorContext::builder(ErrorSeverity::Critical)
            .error_code("ERROR_001")
            .message("Test error")
            .service("test_service")
            .endpoint("test_endpoint")
            .error_source("test_source")
            .request_details("test_details")
            .cause(Some("test cause".to_string()))
            .stack_trace(Some("test stack trace".to_string()))
            .build();

        assert_eq!(
            mailer.build_subject(&context),
            "[PRODUCTION] Critical Alert - ERROR_001 in test_service/test_endpoint"
        );
    }
}
