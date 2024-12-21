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
/// * `service` - Reference to the authentication service
/// * `session` - The current session to delete
///
/// # Returns
/// * `Ok(Empty)` - Empty response on success
/// * `Err(Status)` - Appropriate error status on failure
///
/// # Errors
/// * `INTERNAL_SERVER_ERROR` - Database error
///
/// # Example
/// ```no_run
/// let request = Request::new(Empty {});
/// logout(service, headers, request).await;
/// ```
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
    async fn test_logout_success() {
        let mut mock_db = MockDatabaseOperations::new();
        let session = create_test_session();

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
        let session = create_test_session();

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
