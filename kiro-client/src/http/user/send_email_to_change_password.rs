// http/user/send_email_to_change_password.rs
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

use axum::{extract::State, http::StatusCode, response::IntoResponse, Extension, Json};
use chrono::{Days, Utc};
use kiro_database::{db_bridge::DatabaseOperations, get_env_or};
use kiro_mailer::{ContentType, LinkModel, LinkType, Mailer, MailerTrait};

use crate::{models::UserModel, SessionModel};

/// Password change request handler
///
/// # Description
/// Sends an email to the user to change their password
///
/// # Arguments
/// * `service` - The client service instance
/// * `session` - The current session model
///
/// # Returns
/// * HTTP response with either:
///   * `200 OK` with empty JSON response
///   * Error status code with message
///
/// # Errors
/// * `400 BAD REQUEST` - Invalid password
/// * `404 NOT FOUND` - User not found
/// * `500 INTERNAL SERVER ERROR` - Database or server error
///
/// # Example
///
/// ```rust,no_run
/// use axum::{Extension, extract::State, Json};
/// use http::HeaderMap;
/// use kiro_client::{ClientService, send_email_to_change_password::send_email_to_change_password, SessionModel};
/// use kiro_database::db_bridge::{Database, MockDatabaseOperations};
///
/// // Mock database
/// let mock_db = MockDatabaseOperations::new();
///
/// // Mock service
/// let service = ClientService {
///     db: Database::Mock(mock_db),
/// };
///
/// // Empty headers
/// let headers = HeaderMap::new();
///
/// // Mock session
/// let session = SessionModel::default();
///
/// // Async block to allow `await`
/// tokio::runtime::Runtime::new().unwrap().block_on(async {
///     send_email_to_change_password(&service, &session).await;
///
///     println!("Email change email sent");
/// });
/// ```
#[utoipa::path(
    get,
    path = "/user/send_email_to_change_password",
    tag = "user",
    responses(
        (status = 200, description = "Email change email sent", body = String),
        (status = 404, description = "User not found", body = String),
        (status = 500, description = "Internal server error", body = String)
    )
)]
pub async fn send_email_to_change_password(
    State(service): State<ClientService>, Extension(session): Extension<SessionModel>,
) -> impl IntoResponse {
    // Get user details
    // Get user details
    let user = match service
        .db
        .select::<UserModel>(session.user_id.clone())
        .await
    {
        Ok(Some(user)) => user,
        Ok(None) => {
            return (
                StatusCode::NOT_FOUND,
                Json(serde_json::json!({ "error": "User not found" })),
            )
                .into_response()
        }
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({ "error": format!("Database error: {}", e) })),
            )
                .into_response()
        }
    };

    // Create change email link that expires in 24 hours
    let expiry_time = match Utc::now().checked_add_days(Days::new(1)) {
        Some(expiry) => expiry,
        None => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({ "error": "Failed to calculate expiry time" })),
            )
                .into_response()
        }
    };

    let link =
        match LinkModel::create_from_user(
            &service.db,
            session.user_id.clone(),
            expiry_time,
            LinkType::PasswordChange,
        )
        .await
        .map(|link| link.construct_link())
        {
            Ok(link) => link,
            Err(e) => return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(
                    serde_json::json!({ "error": format!("Failed to create change link: {}", e) }),
                ),
            )
                .into_response(),
        };

    // Send confirmation email
    let template =
        match Mailer::load_template("password_change.html").await {
            Ok(template) => template,
            Err(e) => return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(
                    serde_json::json!({ "error": format!("Failed to load email template: {}", e) }),
                ),
            )
                .into_response(),
        }
        .replace("${{USER_NAME}}", &user.email)
        .replace("${{CHANGE_URL}}", &link);

    let from = get_env_or("SMTP_USER", "test@example.com");

    let message = match Mailer::build_mail(
        &from,
        &user.email,
        "Change Password",
        ContentType::TEXT_HTML,
        template,
    ) {
        Ok(message) => message,
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({ "error": format!("Failed to build email: {}", e) })),
            )
                .into_response()
        }
    };

    match Mailer::new().send_mail(message).await {
        Ok(_) => {}
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({ "error": format!("Failed to send email: {}", e) })),
            )
                .into_response()
        }
    };

    (StatusCode::OK, Json(serde_json::json!({}))).into_response()
}

// TODO: Fix mailer tests Trait, before adding tests
