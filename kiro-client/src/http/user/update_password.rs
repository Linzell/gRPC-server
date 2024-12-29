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

use axum::{extract::State, http::StatusCode, response::IntoResponse, Extension, Json};
#[cfg(feature = "mailer")]
use chrono::{Days, Utc};
use kiro_api::client::v1::UpdatePasswordRequest;
use kiro_database::db_bridge::DatabaseOperations;
#[cfg(feature = "mailer")]
use kiro_database::{get_env_or, DbId};

#[cfg(feature = "mailer")]
use kiro_mailer::{ContentType, LinkModel, LinkType, Mailer, MailerTrait};

use crate::{models::UserModel, SessionModel};

/// User password update route handler
///
/// # Description
/// Updates the current user's password
///
/// # Arguments
/// * `service` - The client service instance
/// * `session` - The current session model
/// * `request` - The password update request
///
/// # Returns
/// * HTTP response with either:
///   * `200 OK` with empty JSON response
///   * Error status code with message
///
/// # Errors
/// * `400 BAD REQUEST` - Invalid password
/// * `404 NOT FOUND` - Link not found
/// * `500 INTERNAL SERVER ERROR` - Database or server error
///
/// # Example
/// ```rust,no_run
/// use axum::{Extension, extract::State, Json};
/// use http::HeaderMap;
/// use kiro_api::client::v1::UpdatePasswordRequest;
/// use kiro_client::{ClientService, update_password::update_password, SessionModel};
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
/// // Mock request
/// let request = UpdatePasswordRequest {
///     temp_token: "temp_token".to_string(),
///     old_password: "old_password".to_string(),
///     password: "new_password".to_string(),
/// };
///
/// // Async block to allow `await`
/// tokio::runtime::Runtime::new().unwrap().block_on(async {
///     update_password(State(service), Extension(session), Json(request)).await;
///
///     println!("Email updated");
/// });
/// ```
#[utoipa::path(
    post,
    path = "/user/update_password",
    tag = "user",
    params(
        UpdatePasswordRequest
    ),
    responses(
        (status = 200, description = "Password updated", body = String),
        (status = 400, description = "Invalid password", body = String),
        (status = 404, description = "Link not found", body = String),
        (status = 500, description = "Internal server error", body = String)

    )
)]
pub async fn update_password(
    State(service): State<ClientService>, Extension(session): Extension<SessionModel>,
    Json(request): Json<UpdatePasswordRequest>,
) -> impl IntoResponse {
    #[cfg(feature = "mailer")]
    {
        // Validate temporary change key
        let link_id = DbId::from(("links".to_string(), request.temp_token.clone()));
        match service.db.select::<LinkModel>(link_id).await {
            Ok(Some(link)) => link,
            Ok(None) => {
                return (
                    StatusCode::NOT_FOUND,
                    Json(serde_json::json!({ "error": "Invalid or expired change key" })),
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
    }

    // Get user info
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

    // Verify old password
    match SessionModel::verify_password(request.old_password, user.password_hash).await {
        Ok(verified) => verified,
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({ "error": e.to_string() })),
            )
                .into_response()
        }
    };

    // Update password hash
    let new_hash = match SessionModel::create_password_hash(request.password.clone()).await {
        Ok(hash) => hash,
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({ "error": e.to_string() })),
            )
                .into_response()
        }
    };

    // Update the password
    match service
        .db
        .update_field(session.user_id.clone(), "/password_hash", new_hash)
        .await
    {
        Ok(_) => (),
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({ "error": e.to_string() })),
            )
                .into_response();
        }
    }

    // Send confirmation email
    #[cfg(feature = "mailer")]
    {
        // Cleanup old password change links
        match LinkModel::delete_link_by_user_and_type(
            &service.db,
            session.user_id.clone(),
            LinkType::PasswordChange,
        )
        .await
        {
            Ok(_) => {}
            Err(e) => return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({ "error": format!("Failed to delete old links: {}", e) })),
            )
                .into_response(),
        };

        // Create reset link for confirmation email
        let expiry = match Utc::now().checked_add_days(Days::new(2)) {
            Some(expiry) => expiry,
            None => {
                return (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(serde_json::json!({ "error": "Failed to calculate expiry" })),
                )
                    .into_response()
            }
        };

        let reset_link = match LinkModel::create_from_user(
            &service.db,
            session.user_id.clone(),
            expiry,
            LinkType::PasswordReset,
        )
        .await
        .map(|link| link.construct_link())
        {
            Ok(link) => link,
            Err(e) => return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({ "error": format!("Failed to create reset link: {}", e) })),
            )
                .into_response(),
        };

        let from = get_env_or("SMTP_USER", "test@example.com");

        // Send confirmation email
        let template = match Mailer::load_template("password_changed.html").await {
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
        .replace("${{RESET_URL}}", &reset_link);

        let message = match Mailer::build_mail(
            &from,
            &user.email,
            "Password Changed",
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
    }

    (StatusCode::OK, Json(serde_json::json!({}))).into_response()
}

// TODO: Fix mailer tests Trait, before adding tests
