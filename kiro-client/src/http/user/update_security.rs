// http/user/update_security.rs
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
use kiro_api::client::v1::UpdateSecurityRequest;
use kiro_database::db_bridge::DatabaseOperations;

use crate::SessionModel;

/// Security update route handler
///
/// # Description
/// Updates the current user's security settings
///
/// # Arguments
/// * `service` - The client service instance
/// * `session` - The current session model
/// * `request` - The security update request
///
/// # Returns
/// * HTTP response with either:
///   * `200 OK` with empty JSON response
///   * Error status code with message
///
/// # Errors
/// * `400 BAD REQUEST` - Invalid security settings
/// * `500 INTERNAL SERVER ERROR` - Database or server error
///
/// # Example
/// ```rust,no_run
/// use axum::{Extension, extract::State, Json};
/// use http::HeaderMap;
/// use kiro_api::client::v1::{update_security_request::Value, UpdateSecurityRequest};
/// use kiro_client::{ClientService, update_security::update_security, SessionModel};
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
/// let request = UpdateSecurityRequest {
///    field: "two_factor".to_string(),
///    value: Some(Value::TwoFactor(true)),
/// };
///
/// // Async block to allow `await`
/// tokio::runtime::Runtime::new().unwrap().block_on(async {
///     update_security(State(service), Extension(session), Json(request)).await;
///
///     println!("Email updated");
/// });
/// ```
#[utoipa::path(
    post,
    path = "/user/update_security",
    tag = "user",
    responses(
        (status = 200, description = "Security settings updated", body = String),
        (status = 400, description = "QR code field is immutable", body = String),
        (status = 400, description = "Invalid security field", body = String),
        (status = 500, description = "Internal server error", body = String)

    )
)]
pub async fn update_security(
    State(service): State<ClientService>, Extension(session): Extension<SessionModel>,
    Json(request): Json<UpdateSecurityRequest>,
) -> impl IntoResponse {
    // QR code field is immutable
    if request.field.as_str() == "qr_code" {
        return (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({ "error": "QR code field is immutable" })),
        )
            .into_response();
    }

    let value = match request.field.as_str() {
        "two_factor" | "magic_link" => serde_json::Value::Bool(request.value.is_some()),
        "qr_code" => serde_json::Value::String(request.value.is_some().to_string()),
        _ => unreachable!(),
    };

    if !["two_factor", "qr_code", "magic_link"].contains(&request.field.as_str()) {
        return (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({ "error": "Invalid security field" })),
        )
            .into_response();
    }
    match service
        .db
        .update_field(
            session.user_id.clone(),
            &format!("settings/security/{}", request.field),
            value,
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

    use kiro_api::client::v1::update_security_request::Value as JsonValue;
    use kiro_database::{db_bridge::MockDatabaseOperations, DatabaseError};
    use mockall::predicate::eq;

    #[tokio::test]
    async fn test_update_security_two_factor_enable() {
        let mut mock_db = MockDatabaseOperations::new();
        let session = SessionModel::default();
        let extension = Extension(session.clone());
        let user_id = session.user_id.clone();

        mock_db
            .expect_update_field()
            .with(
                eq(user_id),
                eq("settings/security/two_factor"),
                eq(serde_json::Value::Bool(true)),
            )
            .times(1)
            .returning(|_, _, _| Ok(()));

        let service = ClientService {
            db: Database::Mock(mock_db),
        };

        let request = UpdateSecurityRequest {
            field: "two_factor".to_string(),
            value: Some(JsonValue::TwoFactor(true)),
        };

        let response = update_security(State(service), extension, Json(request)).await;

        let response = response.into_response();
        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_update_security_two_factor_disable() {
        let mut mock_db = MockDatabaseOperations::new();
        let session = SessionModel::default();
        let extension = Extension(session.clone());
        let user_id = session.user_id.clone();

        mock_db
            .expect_update_field()
            .with(
                eq(user_id),
                eq("settings/security/two_factor"),
                eq(serde_json::Value::Bool(false)),
            )
            .times(1)
            .returning(|_, _, _| Ok(()));

        let service = ClientService {
            db: Database::Mock(mock_db),
        };

        let request = UpdateSecurityRequest {
            field: "two_factor".to_string(),
            value: None,
        };

        let response = update_security(State(service), extension, Json(request)).await;

        let response = response.into_response();
        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_update_security_magic_link_enable() {
        let mut mock_db = MockDatabaseOperations::new();
        let session = SessionModel::default();
        let extension = Extension(session.clone());
        let user_id = session.user_id.clone();

        mock_db
            .expect_update_field()
            .with(
                eq(user_id),
                eq("settings/security/magic_link"),
                eq(serde_json::Value::Bool(true)),
            )
            .times(1)
            .returning(|_, _, _| Ok(()));

        let service = ClientService {
            db: Database::Mock(mock_db),
        };

        let request = UpdateSecurityRequest {
            field: "magic_link".to_string(),
            value: Some(JsonValue::MagicLink(true)),
        };

        let response = update_security(State(service), extension, Json(request)).await;

        let response = response.into_response();
        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_update_security_db_error() {
        let session = SessionModel::default();
        let mut mock_db = MockDatabaseOperations::new();
        let extension = Extension(session.clone());

        mock_db
            .expect_update_field()
            .with(
                eq(session.user_id),
                eq("settings/security/two_factor"),
                eq(serde_json::Value::Bool(true)),
            )
            .times(1)
            .returning(|_, _, _| Err(DatabaseError::Internal("Database error".to_string())));

        let service = ClientService {
            db: Database::Mock(mock_db),
        };

        let request = Json(UpdateSecurityRequest {
            field: "two_factor".to_string(),
            value: Some(JsonValue::TwoFactor(true)),
        });

        let response = update_security(State(service), extension, request).await;

        let response = response.into_response();
        assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);

        let body_bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let error: serde_json::Value = serde_json::from_slice(&body_bytes).unwrap();
        assert_eq!(error["error"], "Internal error: Database error");
    }

    #[tokio::test]
    async fn test_update_security_admin_session() {
        let mut session = SessionModel::default();
        session.is_admin = true;
        let mut mock_db = MockDatabaseOperations::new();
        let extension = Extension(session.clone());
        let user_id = session.user_id.clone();

        mock_db
            .expect_update_field()
            .with(
                eq(user_id),
                eq("settings/security/two_factor"),
                eq(serde_json::Value::Bool(true)),
            )
            .times(1)
            .returning(|_, _, _| Ok(()));

        let service = ClientService {
            db: Database::Mock(mock_db),
        };

        let request = Json(UpdateSecurityRequest {
            field: "two_factor".to_string(),
            value: Some(JsonValue::TwoFactor(true)),
        });

        let response = update_security(State(service), extension, request).await;

        let response = response.into_response();
        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_update_security_toggle_multiple_fields() {
        let mut mock_db = MockDatabaseOperations::new();
        let session = SessionModel::default();
        let extension = Extension(session.clone());
        let user_id = session.user_id.clone();

        mock_db
            .expect_update_field()
            .with(
                eq(user_id),
                eq("settings/security/two_factor"),
                eq(serde_json::Value::Bool(true)),
            )
            .times(1)
            .returning(|_, _, _| Ok(()));

        let service = ClientService {
            db: Database::Mock(mock_db),
        };

        let request = Json(UpdateSecurityRequest {
            field: "two_factor".to_string(),
            value: Some(JsonValue::TwoFactor(true)),
        });

        let response = update_security(State(service), extension, request).await;

        let response = response.into_response();
        assert_eq!(response.status(), StatusCode::OK);

        let mut mock_db = MockDatabaseOperations::new();
        let session = SessionModel::default();
        let extension = Extension(session.clone());
        let user_id = session.user_id.clone();

        mock_db
            .expect_update_field()
            .with(
                eq(user_id),
                eq("settings/security/magic_link"),
                eq(serde_json::Value::Bool(true)),
            )
            .times(1)
            .returning(|_, _, _| Ok(()));

        let service = ClientService {
            db: Database::Mock(mock_db),
        };

        let request = Json(UpdateSecurityRequest {
            field: "magic_link".to_string(),
            value: Some(JsonValue::MagicLink(true)),
        });

        let response = update_security(State(service), extension, request).await;

        let response = response.into_response();
        assert_eq!(response.status(), StatusCode::OK);
    }
}
