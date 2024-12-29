// http/auth/logout.rs
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

use crate::SessionModel;

/// Logout route handler
///
/// # Description
/// Logs out the current user by deleting the session from the database
///
/// # Arguments
/// * `service` - The authentication service instance
/// * `session` - The current session model to terminate
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
/// use kiro_client::{AuthService, logout::logout, SessionModel};
/// use kiro_database::db_bridge::{Database, MockDatabaseOperations};
///
/// // Mock database
/// let mock_db = MockDatabaseOperations::new();
///
/// // Mock service
/// let service = AuthService {
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
///     logout(State(service), Extension(session)).await;
///
///     println!("Logout successful");
/// });
/// ```
#[utoipa::path(
    get,
    path = "/auth/logout",
    tag = "auth",
    responses(
        (status = 200, description = "Session terminated", body = String),
        (status = 500, description = "Internal server error", body = String)
    )
)]
pub async fn logout(
    State(service): State<AuthService>, Extension(session): Extension<SessionModel>,
) -> impl IntoResponse {
    match SessionModel::delete_session(&service.db, session.id).await {
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
    async fn test_logout_success() {
        let mut mock_db = MockDatabaseOperations::new();
        let session = SessionModel::default();

        mock_db
            .expect_delete()
            .with(eq(session.id.clone()))
            .times(1)
            .returning(|_| Ok(Some(())));

        let service = AuthService {
            db: Database::Mock(mock_db),
        };

        let response = logout(State(service), Extension(session)).await;

        let response = response.into_response();
        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_logout_no_session() {
        let mut mock_db = MockDatabaseOperations::new();
        let session = SessionModel::default();

        mock_db
            .expect_delete()
            .with(eq(session.id.clone()))
            .times(1)
            .returning(|_| {
                Err(DatabaseError::Internal(
                    "Authorization flow error".to_string(),
                ))
            });

        let service = AuthService {
            db: Database::Mock(mock_db),
        };

        let response = logout(State(service), Extension(session)).await;

        let response = response.into_response();
        assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);
    }
}
