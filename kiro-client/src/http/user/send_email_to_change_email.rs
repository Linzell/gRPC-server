// http/user/send_email_to_change_email.rs
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

/// Email change request handler
///
/// # Description
/// Sends an email to the user to change their email address
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
/// * `400 BAD REQUEST` - Invalid email address
/// * `401 UNAUTHORIZED` - No session or invalid session
/// * `500 INTERNAL SERVER ERROR` - Database or server error
///
/// # Example
///
/// ```rust,ignore
/// use axum::{Json, Extension};
/// use kiro_api::session::SessionModel;
///
/// let session = SessionModel::default();
/// let response = send_email_to_change_email(&service, &session).await?;
/// ```
pub async fn send_email_to_change_email(
    State(service): State<ClientService>, Extension(session): Extension<SessionModel>,
) -> impl IntoResponse {
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
            LinkType::EmailChange,
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
        match Mailer::load_template("email_change.html").await {
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
        "Change Email",
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
