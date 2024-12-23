// utils/mailer.rs
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

use async_trait::async_trait;
use kiro_database::get_env_or;
use lettre::{
    message::{header::ContentType, IntoBody},
    transport::smtp::{
        authentication::{Credentials, Mechanism},
        response::Response,
    },
    AsyncSmtpTransport, AsyncTransport, Message, Tokio1Executor,
};
use tokio::{fs, io};

#[cfg(any(test, feature = "mock"))]
use mockall::automock;

use crate::error::MailerError;

/// # Mailer
///
/// The mailer module provides utilities for sending emails, including:
/// - SMTP transport configuration
/// - Email template loading
/// - Message construction
/// - Async mail sending
#[derive(Clone)]
pub struct Mailer {
    pub transport: AsyncSmtpTransport<Tokio1Executor>,
}

#[cfg_attr(any(test, feature = "mock"), automock)]
#[async_trait]
pub trait MailerTrait: Send + Sync + Default {
    /// Creates a new instance of the mailer
    fn new() -> Self
    where
        Self: Sized;

    /// Sends an email message asynchronously
    async fn send_mail(&self, message: Message) -> Result<Response, MailerError>;
}

impl Default for Mailer {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl MailerTrait for Mailer {
    fn new() -> Self {
        let smtp_host = get_env_or("SMTP_HOST", "smtp.service.com");
        let smtp_user = get_env_or("SMTP_USER", "test@example.com");
        let smtp_pass = get_env_or("SMTP_PASS", "your_smtp_password");

        let creds = Credentials::new(smtp_user, smtp_pass);

        let smtp_client = AsyncSmtpTransport::<Tokio1Executor>::starttls_relay(&smtp_host)
            .unwrap()
            .credentials(creds)
            .authentication(vec![Mechanism::Plain])
            .build::<Tokio1Executor>();

        Self {
            transport: smtp_client,
        }
    }

    async fn send_mail(&self, message: Message) -> Result<Response, MailerError> {
        self.transport
            .send(message)
            .await
            .map_err(MailerError::SMTP)
    }
}

impl Mailer {
    /// # Load template
    ///
    /// Loads an email template file from the ../email-templates directory.
    ///
    /// ## Arguments
    /// * `name` - The filename of the template to load
    ///
    /// ## Returns
    /// * `io::Result<String>` - The template contents or an IO error
    ///
    /// ## Example
    /// ```rust
    /// let template = mailer.load_template("error.html").await?;
    /// println!("📄 Template: {:?}", template);
    /// ```
    pub async fn load_template(name: &str) -> io::Result<String> {
        let vec = fs::read(format!("./email-templates/{}", name)).await?;
        Ok(String::from_utf8_lossy(&vec).into())
    }

    /// # Build mail
    ///
    /// Constructs an email message with the provided parameters.
    ///
    /// ## Arguments
    /// * `from` - Sender email address
    /// * `to` - Recipient email address
    /// * `subject` - Email subject line
    /// * `content_type` - Content type header (e.g. HTML, plain text)
    /// * `body` - Email body content
    ///
    /// ## Returns
    /// * `Result<Message, MailerError>` - The constructed message or an error
    ///
    /// ## Example
    /// ```rust
    /// let mail = mailer.build_mail(
    ///   "no-reply@test.com",
    ///   "user@example.com",
    ///   "Hello",
    ///   ContentType::HTML,
    ///   "Hello, World!",
    /// )?;
    /// ```
    pub fn build_mail<T>(
        from: &str, to: &str, subject: &str, content_type: ContentType, body: T,
    ) -> Result<Message, MailerError>
    where
        T: IntoBody,
    {
        Ok(Message::builder()
            .from(
                from.parse()
                    .map_err(|_| MailerError::InvalidAddress(to.into()))?,
            )
            .to(to
                .parse()
                .map_err(|_| MailerError::InvalidAddress(to.into()))?)
            .subject(subject)
            .header(content_type)
            .body(body)?)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    use lettre::message::header::{self};
    use std::fs;
    use std::io::Write;

    #[tokio::test]
    async fn test_load_template_success() {
        // Create test template file
        let mut file = fs::File::create("./email-templates/test.html").unwrap();
        file.write_all(b"<h1>Test</h1>\n").unwrap();

        let template = Mailer::load_template("test.html").await.unwrap();
        assert_eq!(template, "<h1>Test</h1>\n");

        // Cleanup
        fs::remove_file("./email-templates/test.html").unwrap();
    }

    #[tokio::test]
    async fn test_load_template_fail() {
        let template = Mailer::load_template("nonexistent.html").await;
        assert!(template.is_err());
    }

    #[tokio::test]
    async fn test_build_mail() {
        let message = Mailer::build_mail::<String>(
            "test@example.com",
            "test@example.com",
            "Test",
            "text/html; charset=utf-8".parse().unwrap(),
            "<h1>Test</h1>".to_string(),
        );

        assert!(message.is_ok());
        let msg = message.unwrap();
        assert_eq!(
            msg.headers().get::<header::Subject>().unwrap().as_ref(),
            "Test"
        );
    }

    #[tokio::test]
    async fn test_send_mail() {
        let message = Mailer::build_mail::<String>(
            "test@example.com",
            "test@example.com",
            "Test",
            "text/html; charset=utf-8".parse().unwrap(),
            "<h1>Test</h1>".to_string(),
        );

        std::env::set_var("SMTP_HOST", "localhost");
        std::env::set_var("SMTP_USER", "test@example.com");
        std::env::set_var("SMTP_PASS", "password");

        let mailer = Mailer::new();
        let response = mailer.send_mail(message.unwrap()).await;
        assert!(response.is_err()); // Expected to fail with mock credentials
    }
}
