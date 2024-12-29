// http/user/update_notifications.rs
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
use kiro_api::client::v1::UpdateNotificationsRequest;
use kiro_database::db_bridge::DatabaseOperations;

use crate::SessionModel;

/// Update notifications route handler
///
/// # Description
/// Updates the current user's notification settings
///
/// # Arguments
/// * `service` - The client service instance
/// * `session` - The current session model
/// * `request` - The notifications update request
///
/// # Returns
/// * HTTP response with either:
///   * `200 OK` with empty JSON response
///   * Error status code with message
///
/// # Errors
/// * `400 BAD REQUEST` - Invalid notification settings
/// * `500 INTERNAL SERVER ERROR` - Database or server error
///
/// # Example
/// ```rust,no_run
/// use axum::{Extension, extract::State, Json};
/// use http::HeaderMap;
/// use kiro_api::client::v1::UpdateNotificationsRequest;
/// use kiro_client::{ClientService, update_notifications::update_notifications, SessionModel};
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
/// let request = UpdateNotificationsRequest {
///     field: "email".to_string(),
///     value: true,
/// };
///
/// // Async block to allow `await`
/// tokio::runtime::Runtime::new().unwrap().block_on(async {
///     update_notifications(State(service), Extension(session), Json(request)).await;
///
///     println!("Notifications updated");
/// });
/// ```
#[utoipa::path(
    post,
    path = "/user/update_notifications",
    tag = "user",
    params(
        UpdateNotificationsRequest
    ),
    responses(
        (status = 200, description = "Notifications updated", body = String),
        (status = 400, description = "Invalid notification settings", body = String),
        (status = 500, description = "Internal server error", body = String)

    )
)]
pub async fn update_notifications(
    State(service): State<ClientService>, Extension(session): Extension<SessionModel>,
    Json(request): Json<UpdateNotificationsRequest>,
) -> impl IntoResponse {
    // Validate the notification field name
    if !["email", "push", "sms"].contains(&request.field.clone().as_str()) {
        return (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({ "error": format!("Invalid notification field: {}", request.field.clone()) })),
        )
            .into_response();
    }

    match service
        .db
        .update_field(
            session.user_id.clone(),
            &format!("settings/notifications/{}", request.field),
            request.value,
        )
        .await
    {
        Ok(_) => (StatusCode::OK, Json(serde_json::json!({}))).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": e.to_string() })),
        )
            .into_response(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use kiro_database::{db_bridge::MockDatabaseOperations, DatabaseError};
    use mockall::predicate::eq;

    #[tokio::test]
    async fn test_update_notifications_email_success() {
        let session = SessionModel::default();
        let mut mock_db = MockDatabaseOperations::new();
        let extension = Extension(session.clone());

        mock_db
            .expect_update_field()
            .with(
                eq(session.user_id.clone()),
                eq("settings/notifications/email"),
                eq(true),
            )
            .times(1)
            .returning(|_, _, _| Ok(()));

        let service = ClientService {
            db: Database::Mock(mock_db),
        };

        let request = UpdateNotificationsRequest {
            field: "email".to_string(),
            value: true,
        };

        let response = update_notifications(State(service), extension, Json(request)).await;

        let response = response.into_response();
        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_update_notifications_push_success() {
        let session = SessionModel::default();
        let mut mock_db = MockDatabaseOperations::new();
        let extension = Extension(session.clone());

        mock_db
            .expect_update_field()
            .with(
                eq(session.user_id.clone()),
                eq("settings/notifications/push"),
                eq(false),
            )
            .times(1)
            .returning(|_, _, _| Ok(()));

        let service = ClientService {
            db: Database::Mock(mock_db),
        };

        let request = UpdateNotificationsRequest {
            field: "push".to_string(),
            value: false,
        };

        let response = update_notifications(State(service), extension, Json(request)).await;

        let response = response.into_response();
        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_update_notifications_sms_success() {
        let session = SessionModel::default();
        let mut mock_db = MockDatabaseOperations::new();
        let extension = Extension(session.clone());

        mock_db
            .expect_update_field()
            .with(
                eq(session.user_id.clone()),
                eq("settings/notifications/sms"),
                eq(true),
            )
            .times(1)
            .returning(|_, _, _| Ok(()));

        let service = ClientService {
            db: Database::Mock(mock_db),
        };

        let request = UpdateNotificationsRequest {
            field: "sms".to_string(),
            value: true,
        };

        let response = update_notifications(State(service), extension, Json(request)).await;

        let response = response.into_response();
        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_update_notifications_invalid_field() {
        let session = SessionModel::default();
        let mock_db = MockDatabaseOperations::new();
        let extension = Extension(session.clone());

        let service = ClientService {
            db: Database::Mock(mock_db),
        };

        let request = UpdateNotificationsRequest {
            field: "invalid_field".to_string(),
            value: true,
        };

        let response = update_notifications(State(service), extension, Json(request)).await;

        let response = response.into_response();
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);

        let body_bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let error: serde_json::Value = serde_json::from_slice(&body_bytes).unwrap();
        assert_eq!(error["error"], "Invalid notification field: invalid_field");
    }

    #[tokio::test]
    async fn test_update_notifications_db_error() {
        let session = SessionModel::default();
        let mut mock_db = MockDatabaseOperations::new();
        let extension = Extension(session.clone());

        mock_db
            .expect_update_field()
            .with(
                eq(session.user_id.clone()),
                eq("settings/notifications/email"),
                eq(true),
            )
            .times(1)
            .returning(|_, _, _| Err(DatabaseError::Internal("Database error".to_string())));

        let service = ClientService {
            db: Database::Mock(mock_db),
        };

        let request = UpdateNotificationsRequest {
            field: "email".to_string(),
            value: true,
        };

        let response = update_notifications(State(service), extension, Json(request)).await;

        let response = response.into_response();
        assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);

        let body_bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let error: serde_json::Value = serde_json::from_slice(&body_bytes).unwrap();
        assert_eq!(error["error"], "Internal error: Database error");
    }

    #[tokio::test]
    async fn test_update_notifications_admin_session() {
        let mut mock_db = MockDatabaseOperations::new();
        let mut admin_session = SessionModel::default();
        admin_session.is_admin = true;
        let user_id = admin_session.user_id.clone();

        mock_db
            .expect_update_field()
            .with(eq(user_id), eq("settings/notifications/email"), eq(true))
            .times(1)
            .returning(|_, _, _| Ok(()));

        let service = ClientService {
            db: Database::Mock(mock_db),
        };

        let request = UpdateNotificationsRequest {
            field: "email".to_string(),
            value: true,
        };

        let response =
            update_notifications(State(service), Extension(admin_session), Json(request)).await;

        let response = response.into_response();
        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_update_notifications_empty_field() {
        let mock_db = MockDatabaseOperations::new();
        let test_session = SessionModel::default();

        let service = ClientService {
            db: Database::Mock(mock_db),
        };

        let request = UpdateNotificationsRequest {
            field: "".to_string(),
            value: true,
        };

        let response =
            update_notifications(State(service), Extension(test_session), Json(request)).await;

        let response = response.into_response();
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);

        let body_bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let error: serde_json::Value = serde_json::from_slice(&body_bytes).unwrap();
        assert_eq!(error["error"], "Invalid notification field: ");
    }
}
