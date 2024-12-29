// services/user/send_email_to_change_email.rs
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

use super::*;

use chrono::{Days, Utc};
use kiro_database::{db_bridge::DatabaseOperations, get_env_or};
use kiro_mailer::{ContentType, LinkModel, LinkType, Mailer, MailerTrait};
use tonic::{Request, Response, Status};

use crate::{models::UserModel, SessionModel};

/// Sends an email to the user to change their email address
///
/// # Arguments
///
/// * `service` - The UserService instance providing database access
/// * `request` - The gRPC request containing the new email address
///
/// # Returns
///
/// * `Ok(Response)` - Email sent successfully
/// * `Err(Status)` - Various error conditions with appropriate status codes
///
/// # Flow
///
/// 1. Validates session and email address
/// 2. Generates a temporary change key
/// 3. Sends an email with a link to confirm the change
///
/// # Errors
///
/// Returns Status::unauthenticated if no valid session is found
/// Returns Status::invalid_argument if no password is provided
/// Returns Status::internal for database errors
///
/// # Example
///
/// ```rust,no_run
/// use tonic::{Request, Response, Status};
/// use kiro_api::{client::v1::client_service_server::ClientService, google::protobuf::Empty};
/// use kiro_database::db_bridge::{Database, MockDatabaseOperations};
///
/// // Mock database
/// let mock_db = MockDatabaseOperations::new();
///
/// // Mock service
/// let service = kiro_client::ClientService {
///     db: Database::Mock(mock_db),
/// };
///
/// // Change email request
/// let request = Request::new(Empty {});
///
/// // Async block to allow `await`
/// tokio::runtime::Runtime::new().unwrap().block_on(async {
///     ClientService::send_email_to_change_email(&service, request).await;
///
///     println!("Email sent successfully");
/// });
/// ```
pub async fn send_email_to_change_email(
    service: &ClientService, request: Request<Empty>,
) -> Result<Response<Empty>, Status> {
    // Get authenticated session from middleware
    let session = request
        .extensions()
        .get::<SessionModel>()
        .ok_or_else(|| Status::unauthenticated("No valid session found"))?;

    // Get user details
    let user = service
        .db
        .select::<UserModel>(session.user_id.clone())
        .await
        .map_err(|e| Status::internal(e.to_string()))?
        .ok_or_else(|| Status::not_found("User not found"))?;

    // Create change email link that expires in 24 hours
    let expiry_time = Utc::now()
        .checked_add_days(Days::new(1))
        .ok_or_else(|| Status::internal("Failed to calculate expiry time"))?;

    let link = LinkModel::create_from_user(
        &service.db,
        session.user_id.clone(),
        expiry_time,
        LinkType::EmailChange,
    )
    .await
    .map_err(|e| Status::internal(format!("Failed to create change link: {}", e)))?
    .construct_link();

    // Load and populate email template
    let template = Mailer::load_template("email_change.html")
        .await
        .map_err(|e| Status::internal(format!("Failed to load email template: {}", e)))?
        .replace("${{USER_NAME}}", &user.email)
        .replace("${{CHANGE_URL}}", &link);

    let from = get_env_or("SMTP_USER", "test@example.com");

    // Build and send email
    let message = Mailer::build_mail(
        &from,
        &user.email,
        "Change Email",
        ContentType::TEXT_HTML,
        template,
    )
    .map_err(|e| Status::internal(format!("Failed to build email: {}", e)))?;

    Mailer::new()
        .send_mail(message)
        .await
        .map_err(|e| Status::internal(format!("Failed to send email: {}", e)))?;

    Ok(Response::new(Empty {}))
}

// TODO: Fix mailer tests Trait, before adding tests
