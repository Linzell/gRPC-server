// http/user/update_language.rs
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
use kiro_api::client::v1::UpdateLanguageRequest;
use kiro_database::db_bridge::DatabaseOperations;

use crate::SessionModel;

/// Language update route handler
///
/// # Description
/// Updates the current user's language preference
///
/// # Arguments
/// * `service` - The client service instance
/// * `session` - The current session model
/// * `request` - The language update request
///
/// # Returns
/// * HTTP response with either:
///   * `200 OK` with empty JSON response
///   * Error status code with message
///
/// # Errors
/// * `400 BAD REQUEST` - Invalid language
/// * `401 UNAUTHORIZED` - No session or invalid session
/// * `500 INTERNAL SERVER ERROR` - Database or server error
///
/// # Example
/// ```rust,ignore
/// use axum::{Json, Extension};
/// use kiro_api::session::SessionModel;
/// use kiro_api::user::UpdateLanguageRequest;
///
/// let session = SessionModel::default();
/// let request = UpdateLanguageRequest {
///     language: "fr".to_string(),
/// };
/// let response = update_language(State(service), Extension(session), Json(request)).await;
/// ```
pub async fn update_language(
    State(service): State<ClientService>, Extension(session): Extension<SessionModel>,
    Json(request): Json<UpdateLanguageRequest>,
) -> impl IntoResponse {
    // Check if the language is a valid Language enum value
    if !matches!(request.language.clone(), 0..=9) {
        return (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({ "error": "Invalid language value" })),
        )
            .into_response();
    }

    match service
        .db
        .update_field(
            session.user_id.clone(),
            "language",
            request.language.clone(),
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

    use chrono::Utc;
    use kiro_database::{db_bridge::MockDatabaseOperations, DatabaseError, DbDateTime, DbId};
    use mockall::predicate::eq;

    fn create_test_session() -> SessionModel {
        SessionModel {
            id: DbId::from(("sessions", "1")),
            session_key: "session_token".to_string(),
            expires_at: DbDateTime::from(Utc::now() + chrono::Duration::days(2)),
            user_id: DbId::from(("users", "1")),
            ip_address: Some("127.0.0.1".to_string()),
            is_admin: false,
        }
    }

    #[tokio::test]
    async fn test_update_language_success() {
        let mut mock_db = MockDatabaseOperations::new();
        let session = create_test_session();
        let extension = Extension(session.clone());
        let user_id = session.user_id.clone();

        mock_db
            .expect_update_field()
            .with(eq(user_id), eq("language"), eq(0))
            .times(1)
            .returning(|_, _, _| Ok(()));

        let service = ClientService {
            db: Database::Mock(mock_db),
        };

        let request = Json(UpdateLanguageRequest { language: 0 });

        let response = update_language(State(service), extension, request).await;

        let response = response.into_response();
        assert!(response.status().is_success());
    }

    #[tokio::test]
    async fn test_update_language_db_error() {
        let session = create_test_session();
        let mut mock_db = MockDatabaseOperations::new();
        let extension = Extension(session.clone());

        mock_db
            .expect_update_field()
            .with(eq(session.user_id), eq("language"), eq(0))
            .times(1)
            .returning(|_, _, _| Err(DatabaseError::Internal("Database error".to_string())));

        let service = ClientService {
            db: Database::Mock(mock_db),
        };

        let request = Json(UpdateLanguageRequest { language: 0 });

        let response = update_language(State(service), extension, request).await;

        let response = response.into_response();
        assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);

        let body_bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let error: serde_json::Value = serde_json::from_slice(&body_bytes).unwrap();
        assert_eq!(error["error"], "Internal error: Database error");
    }

    #[tokio::test]
    async fn test_update_language_admin_session() {
        let mut session = create_test_session();
        session.is_admin = true;
        let mut mock_db = MockDatabaseOperations::new();
        let extension = Extension(session.clone());
        let user_id = session.user_id.clone();

        mock_db
            .expect_update_field()
            .with(eq(user_id), eq("language"), eq(0))
            .times(1)
            .returning(|_, _, _| Ok(()));

        let service = ClientService {
            db: Database::Mock(mock_db),
        };

        let request = Json(UpdateLanguageRequest { language: 0 });

        let response = update_language(State(service), extension, request).await;

        let response = response.into_response();
        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_update_language_invalid_language() {
        let session = create_test_session();
        let mock_db = MockDatabaseOperations::new();
        let extension = Extension(session.clone());

        let service = ClientService {
            db: Database::Mock(mock_db),
        };

        let request = Json(UpdateLanguageRequest { language: 99 });

        let response = update_language(State(service), extension, request).await;

        let response = response.into_response();
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);

        let body_bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let error: serde_json::Value = serde_json::from_slice(&body_bytes).unwrap();
        assert_eq!(error["error"], "Invalid language value");
    }
}
