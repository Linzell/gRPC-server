// http/user/disable_user.rs
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

/// Disable user route handler
///
/// # Description
/// Disables the current user account
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
/// ```rust,no_run
/// use axum::{Extension, extract::State, Json};
/// use http::HeaderMap;
/// use kiro_client::{ClientService, disable_user::disable_user, SessionModel};
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
///     disable_user(State(service), Extension(session)).await;
///
///     println!("User deleted");
/// });
/// ```
#[utoipa::path(
    delete,
    path = "/user/disable_user",
    tag = "user",
    responses(
        (status = 200, description = "User disabled", body = String),
        (status = 500, description = "Internal server error", body = String)
    )
)]
pub async fn disable_user(
    State(service): State<ClientService>, Extension(session): Extension<SessionModel>,
) -> impl IntoResponse {
    match service.db.delete_soft(session.user_id.clone()).await {
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
    async fn test_disable_user_success() {
        let session = SessionModel::default();
        let mut mock_db = MockDatabaseOperations::new();
        let extension = Extension(session.clone());

        mock_db
            .expect_delete_soft()
            .with(eq(session.user_id.clone()))
            .times(1)
            .returning(|_| Ok(()));

        let service = ClientService {
            db: Database::Mock(mock_db),
        };

        let response = disable_user(State(service), extension).await;

        let response = response.into_response();
        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_disable_user_db_error() {
        let session = SessionModel::default();
        let mut mock_db = MockDatabaseOperations::new();
        let extension = Extension(session);

        mock_db
            .expect_delete_soft()
            .times(1)
            .returning(|_| Err(DatabaseError::Internal("Database error".to_string())));

        let service = ClientService {
            db: Database::Mock(mock_db),
        };

        let response = disable_user(State(service), extension).await;

        let response = response.into_response();
        assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);
    }

    #[tokio::test]
    async fn test_disable_user_admin_session() {
        let mut session = SessionModel::default();
        let mut mock_db = MockDatabaseOperations::new();
        let extension = Extension(session.clone());

        session.is_admin = true;

        mock_db
            .expect_delete_soft()
            .with(eq(session.user_id.clone()))
            .times(1)
            .returning(|_| Ok(()));

        let service = ClientService {
            db: Database::Mock(mock_db),
        };

        let response = disable_user(State(service), extension).await;

        let response = response.into_response();
        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_disable_user_already_disabled() {
        let session = SessionModel::default();
        let mut mock_db = MockDatabaseOperations::new();
        let extension = Extension(session.clone());

        mock_db
            .expect_delete_soft()
            .with(eq(session.user_id.clone()))
            .times(1)
            .returning(|_| Ok(()));

        let service = ClientService {
            db: Database::Mock(mock_db),
        };

        let response = disable_user(State(service), extension).await;

        let response = response.into_response();
        assert_eq!(response.status(), StatusCode::OK);
    }
}
