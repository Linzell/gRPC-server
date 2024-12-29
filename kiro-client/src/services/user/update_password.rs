// services/user/update_password.rs
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

/// Updates a user's password and sends a confirmation email
///
/// # Arguments
///
/// * `service` - The UserService instance containing database access
/// * `request` - gRPC request containing:
///   * `temp_key` - Temporary key for password reset verification
///   * `old_password` - Current password for verification
///   * `new_password` - New password to set
///
/// # Returns
///
/// * `Ok(Response)` - Password updated successfully
/// * `Err(Status)` - Various error conditions with appropriate status codes
///
/// # Flow
/// 1. Validates session and temporary key
/// 2. Verifies old password
/// 3. Updates password hash
/// 4. Sends confirmation email with reset link
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
/// use kiro_api::client::v1::{client_service_server::ClientService, UpdatePasswordRequest};
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
/// // Update password request
/// let request = Request::new(UpdatePasswordRequest {
///     temp_token: "temp_token".to_string(),
///     old_password: "old_password".to_string(),
///     password: "new_password".to_string(),
/// });
///
///
/// // Async block to allow `await`
/// tokio::runtime::Runtime::new().unwrap().block_on(async {
///     ClientService::update_password(&service, request).await;
///
///     println!("Password updated");
/// });
/// ```
pub async fn update_password(
    service: &ClientService, request: Request<UpdatePasswordRequest>,
) -> Result<Response<Empty>, Status> {
    // Get session from middleware
    let session = request
        .extensions()
        .get::<SessionModel>()
        .ok_or_else(|| Status::unauthenticated("No valid session found"))?;

    let request = request.get_ref();

    #[cfg(feature = "mailer")]
    {
        // Validate temporary key
        let link_id = DbId::from(("links".to_string(), request.temp_token.clone()));

        match service.db.select::<LinkModel>(link_id.clone()).await {
            Ok(Some(link)) => link,
            Ok(None) => return Err(Status::not_found("Link not found")),
            Err(e) => return Err(Status::internal(e.to_string())),
        };
    }

    // Get user info
    let user = match service
        .db
        .select::<UserModel>(session.user_id.clone())
        .await
    {
        Ok(Some(user)) => user,
        Ok(None) => return Err(Status::not_found("User not found")),
        Err(e) => return Err(Status::internal(e.to_string())),
    };

    // Verify old password
    if !SessionModel::verify_password(request.old_password.clone(), user.password_hash).await? {
        return Err(Status::invalid_argument("Invalid password"));
    }

    // Update password hash
    let new_hash = SessionModel::create_password_hash(request.password.clone()).await?;

    service
        .db
        .update_field(session.user_id.clone(), "/password_hash", new_hash)
        .await
        .map_err(|e| Status::internal(e.to_string()))?;

    #[cfg(feature = "mailer")]
    {
        // Clean up old password change link
        LinkModel::delete_link_by_user_and_type(
            &service.db,
            session.user_id.clone(),
            LinkType::PasswordChange,
        )
        .await?;

        // Create reset link for confirmation email
        let expiry = Utc::now()
            .checked_add_days(Days::new(2))
            .ok_or_else(|| Status::internal("Failed to calculate expiry date"))?;

        let reset_link = LinkModel::create_from_user(
            &service.db,
            session.user_id.clone(),
            expiry,
            LinkType::PasswordReset,
        )
        .await
        .map(|link| link.construct_link())
        .map_err(|e| Status::internal(e.to_string()))?;

        let from = get_env_or("SMTP_USER", "test@example.com");

        // Send confirmation email
        let template = Mailer::load_template("password_changed.html")
            .await
            .map_err(|e| Status::internal(e.to_string()))?
            .replace("${{USER_NAME}}", &user.email)
            .replace("${{RESET_URL}}", &reset_link);

        let message = Mailer::build_mail(
            &from,
            &user.email,
            "Password Changed",
            ContentType::TEXT_HTML,
            template,
        )
        .map_err(|e| Status::internal(e.to_string()))?;

        Mailer::new()
            .send_mail(message)
            .await
            .map_err(|e| Status::internal(e.to_string()))?;
    }

    Ok(Response::new(Empty {}))
}

// TODO: Fix mailer tests Trait, before adding tests
