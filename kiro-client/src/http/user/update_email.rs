// http/user/update_email.rs
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
use kiro_api::client::v1::UpdateEmailRequest;
use kiro_database::db_bridge::DatabaseOperations;
#[cfg(feature = "mailer")]
use kiro_database::{get_env_or, DbId};

#[cfg(feature = "mailer")]
use kiro_mailer::{ContentType, LinkModel, LinkType, Mailer, MailerTrait};

use crate::{models::UserModel, SessionModel};

/// User email update route handler
///
/// # Description
/// Updates the current user's email address
///
/// # Arguments
/// * `service` - The client service instance
/// * `session` - The current session model
/// * `request` - The email update request
///
/// # Returns
/// * HTTP response with either:
///   * `200 OK` with empty JSON response
///   * Error status code with message
///
/// # Errors
/// * `400 BAD REQUEST` - Invalid email address
/// * `404 NOT FOUND` - Link not found
/// * `500 INTERNAL SERVER ERROR` - Database or server error
///
/// # Example
/// ```rust,no_run
/// use axum::{Extension, extract::State, Json};
/// use http::HeaderMap;
/// use kiro_api::client::v1::UpdateEmailRequest;
/// use kiro_client::{ClientService, update_email::update_email, SessionModel};
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
/// let request = UpdateEmailRequest {
///     email: "test@test.com".to_string(),
///     temp_token: "temp_token".to_string(),
/// };
///
/// // Async block to allow `await`
/// tokio::runtime::Runtime::new().unwrap().block_on(async {
///     update_email(State(service), Extension(session), Json(request)).await;
///
///     println!("Email updated");
/// });
/// ```
#[utoipa::path(
    post,
    path = "/user/update_email",
    tag = "user",
    params(
        UpdateEmailRequest
    ),
    responses(
        (status = 200, description = "Email updated", body = String),
        (status = 400, description = "Invalid email address", body = String),
        (status = 404, description = "Link not found", body = String),
        (status = 500, description = "Internal server error", body = String)

    )
)]
pub async fn update_email(
    State(service): State<ClientService>, Extension(session): Extension<SessionModel>,
    Json(request): Json<UpdateEmailRequest>,
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

    // Check if email is already used
    match UserModel::check_email(&service.db, request.email.clone()).await {
        Ok(true) => {}
        Ok(false) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(serde_json::json!({ "error": "Email already in use" })),
            )
                .into_response()
        }
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({ "error": e.to_string() })),
            )
                .into_response()
        }
    };

    // Update email
    match service
        .db
        .update_field(session.user_id.clone(), "email", request.email.clone())
        .await
    {
        Ok(_) => {}
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({ "error": format!("Failed to update email: {}", e) })),
            )
                .into_response()
        }
    };

    #[cfg(feature = "mailer")]
    {
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

        // Cleanup old change links
        match LinkModel::delete_link_by_user_and_type(
            &service.db,
            session.user_id.clone(),
            LinkType::EmailChange,
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

        // Generate reset link for safety
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
            LinkType::EmailReset,
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

        // Send confirmation email
        let template = match Mailer::load_template("email_changed.html").await {
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
        .replace("${{RESET_URL}}", &reset_link)
        .replace("${{NEW_MAIL}}", &request.email);

        let from = get_env_or("SMTP_USER", "test@example.com");

        let message = match Mailer::build_mail(
            &from,
            &user.email,
            "Email Changed",
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
