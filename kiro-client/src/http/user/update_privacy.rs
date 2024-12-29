// http/user/update_privacy.rs
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
use kiro_api::client::v1::UpdatePrivacyRequest;
use kiro_database::db_bridge::DatabaseOperations;

use crate::SessionModel;

/// Privacy update route handler
///
/// # Description
/// Updates the current user's privacy settings
///
/// # Arguments
/// * `service` - The client service instance
/// * `session` - The current session model
/// * `request` - The privacy update request
///
/// # Returns
/// * HTTP response with either:
///   * `200 OK` with empty JSON response
///   * Error status code with message
///
/// # Errors
/// * `400 BAD REQUEST` - Invalid privacy settings
/// * `500 INTERNAL SERVER ERROR` - Database or server error
///
/// # Example
/// ```rust,no_run
/// use axum::{Extension, extract::State, Json};
/// use http::HeaderMap;
/// use kiro_api::client::v1::UpdatePrivacyRequest;
/// use kiro_client::{ClientService, update_privacy::update_privacy, SessionModel};
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
/// let request = UpdatePrivacyRequest {
///    field: "data_collection".to_string(),
///    value: true,
/// };
///
/// // Async block to allow `await`
/// tokio::runtime::Runtime::new().unwrap().block_on(async {
///     update_privacy(State(service), Extension(session), Json(request)).await;
///
///     println!("Privacy updated");
/// });
/// ```
#[utoipa::path(
    post,
    path = "/user/update_email",
    tag = "user",
    params(
        UpdatePrivacyRequest
    ),
    responses(
        (status = 200, description = "Privacy updated", body = String),
        (status = 400, description = "Invalid privacy settings", body = String),
        (status = 500, description = "Internal server error", body = String)

    )
)]
pub async fn update_privacy(
    State(service): State<ClientService>, Extension(session): Extension<SessionModel>,
    Json(request): Json<UpdatePrivacyRequest>,
) -> impl IntoResponse {
    // Validate the privacy field name
    if !["data_collection", "location"].contains(&request.field.clone().as_str()) {
        return (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({ "error": format!("Invalid privacy field: {}", request.field.clone()) })),
        )
            .into_response();
    }

    match service
        .db
        .update_field(
            session.user_id.clone(),
            &format!("settings/privacy/{}", request.field),
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
    async fn test_update_privacy_data_collection_success() {
        let session = SessionModel::default();
        let mut mock_db = MockDatabaseOperations::new();
        let extension = Extension(session.clone());

        mock_db
            .expect_update_field()
            .with(
                eq(session.user_id.clone()),
                eq("settings/privacy/data_collection"),
                eq(true),
            )
            .times(1)
            .returning(|_, _, _| Ok(()));

        let service = ClientService {
            db: Database::Mock(mock_db),
        };

        let request = UpdatePrivacyRequest {
            field: "data_collection".to_string(),
            value: true,
        };
        let response = update_privacy(State(service), extension, Json(request)).await;

        let response = response.into_response();
        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_update_privacy_location_success() {
        let session = SessionModel::default();
        let mut mock_db = MockDatabaseOperations::new();
        let extension = Extension(session.clone());

        mock_db
            .expect_update_field()
            .with(
                eq(session.user_id.clone()),
                eq("settings/privacy/location"),
                eq(true),
            )
            .times(1)
            .returning(|_, _, _| Ok(()));

        let service = ClientService {
            db: Database::Mock(mock_db),
        };

        let request = UpdatePrivacyRequest {
            field: "location".to_string(),
            value: true,
        };
        let response = update_privacy(State(service), extension, Json(request)).await;

        let response = response.into_response();
        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_update_privacy_invalid_field() {
        let session = SessionModel::default();
        let extension = Extension(session.clone());

        let service = ClientService {
            db: Database::Mock(MockDatabaseOperations::new()),
        };

        let request = UpdatePrivacyRequest {
            field: "invalid_field".to_string(),
            value: true,
        };
        let response = update_privacy(State(service), extension, Json(request)).await;

        let response = response.into_response();
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);

        let body_bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let error: serde_json::Value = serde_json::from_slice(&body_bytes).unwrap();
        assert_eq!(error["error"], "Invalid privacy field: invalid_field");
    }

    #[tokio::test]
    async fn test_update_privacy_db_error() {
        let session = SessionModel::default();
        let mut mock_db = MockDatabaseOperations::new();
        let extension = Extension(session.clone());

        mock_db
            .expect_update_field()
            .with(
                eq(session.user_id.clone()),
                eq("settings/privacy/data_collection"),
                eq(true),
            )
            .times(1)
            .returning(|_, _, _| Err(DatabaseError::Internal("Database error".to_string())));

        let service = ClientService {
            db: Database::Mock(mock_db),
        };

        let request = UpdatePrivacyRequest {
            field: "data_collection".to_string(),
            value: true,
        };
        let response = update_privacy(State(service), extension, Json(request)).await;

        let response = response.into_response();
        assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);

        let body_bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let error: serde_json::Value = serde_json::from_slice(&body_bytes).unwrap();
        assert_eq!(error["error"], "Internal error: Database error");
    }

    #[tokio::test]
    async fn test_update_privacy_admin_session() {
        let mut session = SessionModel::default();
        session.is_admin = true;
        let mut mock_db = MockDatabaseOperations::new();
        let extension = Extension(session.clone());

        mock_db
            .expect_update_field()
            .with(
                eq(session.user_id.clone()),
                eq("settings/privacy/data_collection"),
                eq(true),
            )
            .times(1)
            .returning(|_, _, _| Ok(()));

        let service = ClientService {
            db: Database::Mock(mock_db),
        };

        let request = UpdatePrivacyRequest {
            field: "data_collection".to_string(),
            value: true,
        };
        let response = update_privacy(State(service), extension, Json(request)).await;

        let response = response.into_response();
        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_update_privacy_empty_field() {
        let session = SessionModel::default();
        let extension = Extension(session.clone());

        let service = ClientService {
            db: Database::Mock(MockDatabaseOperations::new()),
        };

        let request = UpdatePrivacyRequest {
            field: "".to_string(),
            value: true,
        };
        let response = update_privacy(State(service), extension, Json(request)).await;

        let response = response.into_response();
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);

        let body_bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let error: serde_json::Value = serde_json::from_slice(&body_bytes).unwrap();
        assert_eq!(error["error"], "Invalid privacy field: ");
    }

    #[tokio::test]
    async fn test_update_privacy_toggle_both_fields() {
        let session = SessionModel::default();
        let mut mock_db = MockDatabaseOperations::new();
        let extension = Extension(session.clone());

        mock_db
            .expect_update_field()
            .with(
                eq(session.user_id.clone()),
                eq("settings/privacy/data_collection"),
                eq(true),
            )
            .times(1)
            .returning(|_, _, _| Ok(()));

        let service = ClientService {
            db: Database::Mock(mock_db),
        };

        let request = UpdatePrivacyRequest {
            field: "data_collection".to_string(),
            value: true,
        };
        let response = update_privacy(State(service), extension, Json(request)).await;

        let response = response.into_response();
        assert_eq!(response.status(), StatusCode::OK);

        let session = SessionModel::default();
        let mut mock_db = MockDatabaseOperations::new();
        let extension = Extension(session.clone());

        mock_db
            .expect_update_field()
            .with(
                eq(session.user_id.clone()),
                eq("settings/privacy/location"),
                eq(false),
            )
            .times(1)
            .returning(|_, _, _| Ok(()));

        let service = ClientService {
            db: Database::Mock(mock_db),
        };

        let request = UpdatePrivacyRequest {
            field: "location".to_string(),
            value: false,
        };
        let response = update_privacy(State(service), extension, Json(request)).await;

        let response = response.into_response();
        assert_eq!(response.status(), StatusCode::OK);
    }
}
