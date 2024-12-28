// http/user/delete_user.rs
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
use kiro_database::db_bridge::DatabaseOperations;

use crate::SessionModel;

/// Delete user route handler
///
/// # Description
/// Deletes the current user account
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
/// * `500 INTERNAL SERVER ERROR` - Database or server error
///
/// # Example
/// ```rust,ignore
/// use axum::{Json, Extension};
/// use kiro_api::session::SessionModel;
///
/// let session = SessionModel::default();
/// let response = delete_user(State(service), Extension(session)).await;
/// ```
pub async fn delete_user(
    State(service): State<ClientService>, Extension(session): Extension<SessionModel>,
) -> impl IntoResponse {
    match service.db.delete(session.user_id.clone()).await {
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
    async fn test_delete_user_success() {
        let session = create_test_session();
        let mut mock_db = MockDatabaseOperations::new();
        let extension = Extension(session.clone());

        mock_db
            .expect_delete()
            .with(eq(session.user_id.clone()))
            .times(1)
            .returning(|_| Ok(Some(())));

        let service = ClientService {
            db: Database::Mock(mock_db),
        };

        let response = delete_user(State(service), extension).await;

        let response = response.into_response();
        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_delete_user_db_error() {
        let session = create_test_session();
        let mut mock_db = MockDatabaseOperations::new();
        let extension = Extension(session.clone());

        mock_db
            .expect_delete()
            .with(eq(session.user_id.clone()))
            .times(1)
            .returning(|_| Err(DatabaseError::Internal("Database error".to_string())));

        let service = ClientService {
            db: Database::Mock(mock_db),
        };

        let response = delete_user(State(service), extension).await;

        let response = response.into_response();
        assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);
    }

    #[tokio::test]
    async fn test_delete_user_admin_session() {
        let mut session = create_test_session();
        session.is_admin = true;
        let mut mock_db = MockDatabaseOperations::new();
        let extension = Extension(session.clone());

        mock_db
            .expect_delete()
            .with(eq(session.user_id.clone()))
            .times(1)
            .returning(|_| Ok(Some(())));

        let service = ClientService {
            db: Database::Mock(mock_db),
        };

        let response = delete_user(State(service), extension).await;

        let response = response.into_response();
        assert_eq!(response.status(), StatusCode::OK);
    }
}
