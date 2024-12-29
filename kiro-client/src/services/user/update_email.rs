// services/user/update_email.rs
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

#[cfg(feature = "mailer")]
use chrono::{Days, Utc};
use kiro_database::db_bridge::DatabaseOperations;
#[cfg(feature = "mailer")]
use kiro_database::{get_env_or, DbId};
use tonic::{Request, Response, Status};

#[cfg(feature = "mailer")]
use kiro_mailer::{ContentType, LinkModel, LinkType, Mailer, MailerTrait};

use crate::{models::UserModel, SessionModel};

/// Updates a user's email address with validation and notification
///
/// # Arguments
///
/// * `service` - The UserService instance providing database access
/// * `request` - The gRPC request containing temp_key and new email
///
/// # Returns
///
/// * `Ok(Response)` - Email update successful
/// * `Err(Status)` - Various error conditions with appropriate status codes
///
/// # Flow
///
/// 1. Validates session and temporary change key
/// 2. Checks if email is available
/// 3. Updates user's email
/// 4. Sends confirmation email with reset option
/// 5. Cleans up temporary links
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
/// use kiro_api::client::v1::{client_service_server::ClientService, UpdateEmailRequest};
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
/// // Update email request
/// let request = Request::new(UpdateEmailRequest {
///     email: "test@test.com".to_string(),
///     temp_token: "temp_token".to_string(),
/// });
///
/// // Async block to allow `await`
/// tokio::runtime::Runtime::new().unwrap().block_on(async {
///     ClientService::update_email(&service, request).await;
///
///     println!("Email updated");
/// });
/// ```
pub async fn update_email(
    service: &ClientService, request: Request<UpdateEmailRequest>,
) -> Result<Response<Empty>, Status> {
    // Get authenticated session
    let session = request
        .extensions()
        .get::<SessionModel>()
        .ok_or_else(|| Status::unauthenticated("No valid session found"))?;

    let request = request.get_ref();

    #[cfg(feature = "mailer")]
    {
        // Validate temporary change key
        let link_id = DbId::from(("links".to_string(), request.temp_token.clone()));
        match service.db.select::<LinkModel>(link_id).await {
            Ok(Some(link)) => link,
            Ok(None) => return Err(Status::not_found("Invalid or expired change key")),
            Err(e) => return Err(Status::internal(format!("Database error: {}", e))),
        };
    }

    // Check if email is already used
    if UserModel::check_email(&service.db, request.email.clone()).await? {
        return Err(Status::already_exists("Email address already in use"));
    }

    // Update email
    service
        .db
        .update_field(session.user_id.clone(), "email", request.email.clone())
        .await
        .map_err(|e| Status::internal(format!("Failed to update email: {}", e)))?;

    #[cfg(feature = "mailer")]
    {
        // Get user details
        let user = match service
            .db
            .select::<UserModel>(session.user_id.clone())
            .await
        {
            Ok(Some(user)) => user,
            Ok(None) => return Err(Status::not_found("User not found")),
            Err(e) => return Err(Status::internal(format!("Database error: {}", e))),
        };

        // Cleanup old change links
        LinkModel::delete_link_by_user_and_type(
            &service.db,
            session.user_id.clone(),
            LinkType::EmailChange,
        )
        .await?;

        // Generate reset link for safety
        let expiry = Utc::now()
            .checked_add_days(Days::new(2))
            .ok_or_else(|| Status::internal("Failed to calculate expiry"))?;

        let reset_link = LinkModel::create_from_user(
            &service.db,
            session.user_id.clone(),
            expiry,
            LinkType::EmailReset,
        )
        .await
        .map(|link| link.construct_link())
        .map_err(|e| Status::internal(format!("Failed to create reset link: {}", e)))?;

        // Send confirmation email
        let template = Mailer::load_template("email_changed.html")
            .await
            .map_err(|e| Status::internal(format!("Failed to load email template: {}", e)))?
            .replace("${{USER_NAME}}", &user.email)
            .replace("${{RESET_URL}}", &reset_link)
            .replace("${{NEW_MAIL}}", &request.email);

        let from = get_env_or("SMTP_USER", "test@example.com");

        let message = Mailer::build_mail(
            &from,
            &user.email,
            "Email Changed",
            ContentType::TEXT_HTML,
            template,
        )
        .map_err(|e| Status::internal(format!("Failed to build email: {}", e)))?;

        Mailer::new()
            .send_mail(message)
            .await
            .map_err(|e| Status::internal(format!("Failed to send email: {}", e)))?;
    }

    Ok(Response::new(Empty {}))
}

// TODO: Fix mailer tests Trait, before adding tests
