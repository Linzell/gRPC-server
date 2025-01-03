// http/user/update_theme.rs
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
use kiro_api::client::v1::UpdateThemeRequest;
use kiro_database::db_bridge::DatabaseOperations;

use crate::SessionModel;

/// User theme update route handler
///
/// # Description
/// Updates the current user's theme preference
///
/// # Arguments
/// * `service` - The client service instance
/// * `session` - The current session model
/// * `request` - The theme update request
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
///
/// ```rust,no_run
/// use axum::{Extension, extract::State, Json};
/// use http::HeaderMap;
/// use kiro_api::client::v1::UpdateThemeRequest;
/// use kiro_client::{ClientService, update_theme::update_theme, SessionModel};
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
/// let request = UpdateThemeRequest {
///     theme: 0,   // Assuming 0 represents light theme
/// };
///
/// // Async block to allow `await`
/// tokio::runtime::Runtime::new().unwrap().block_on(async {
///     update_theme(State(service), Extension(session), Json(request)).await;
///
///     println!("Theme updated");
/// });
/// ```
#[utoipa::path(
    post,
    path = "/user/update_theme",
    tag = "user",
    params(
        UpdateThemeRequest
    ),
    responses(
        (status = 200, description = "Theme updated", body = String),
        (status = 500, description = "Internal server error", body = String)

    )
)]
pub async fn update_theme(
    State(service): State<ClientService>, Extension(session): Extension<SessionModel>,
    Json(request): Json<UpdateThemeRequest>,
) -> impl IntoResponse {
    // Update the user's theme
    match service
        .db
        .update_field(session.user_id.clone(), "theme", request.theme)
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
    async fn test_update_theme_light_success() {
        let mut mock_db = MockDatabaseOperations::new();
        let test_session = SessionModel::default();
        let extension = Extension(test_session.clone());
        let user_id = test_session.user_id.clone();

        mock_db
            .expect_update_field()
            .with(eq(user_id), eq("theme"), eq(0)) // Assuming 0 represents light theme
            .times(1)
            .returning(|_, _, _| Ok(()));

        let service = ClientService {
            db: Database::Mock(mock_db),
        };

        let request = Json(UpdateThemeRequest { theme: 0 });

        let response = update_theme(State(service), extension, request).await;

        let response = response.into_response();
        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_update_theme_dark_success() {
        let mut mock_db = MockDatabaseOperations::new();
        let test_session = SessionModel::default();
        let extension = Extension(test_session.clone());
        let user_id = test_session.user_id.clone();

        mock_db
            .expect_update_field()
            .with(eq(user_id), eq("theme"), eq(1)) // Assuming 1 represents dark theme
            .times(1)
            .returning(|_, _, _| Ok(()));

        let service = ClientService {
            db: Database::Mock(mock_db),
        };

        let request = Json(UpdateThemeRequest { theme: 1 });

        let response = update_theme(State(service), extension, request).await;

        let response = response.into_response();
        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_update_theme_db_error() {
        let mut mock_db = MockDatabaseOperations::new();
        let test_session = SessionModel::default();
        let extension = Extension(test_session.clone());
        let user_id = test_session.user_id.clone();

        mock_db
            .expect_update_field()
            .with(eq(user_id), eq("theme"), eq(0)) // Assuming 0 represents light theme
            .times(1)
            .returning(|_, _, _| Err(DatabaseError::Internal("Database error".to_string())));

        let service = ClientService {
            db: Database::Mock(mock_db),
        };

        let request = Json(UpdateThemeRequest { theme: 0 });

        let response = update_theme(State(service), extension, request).await;

        let response = response.into_response();
        assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);

        let body_bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let error: serde_json::Value = serde_json::from_slice(&body_bytes).unwrap();
        assert_eq!(error["error"], "Internal error: Database error");
    }

    #[tokio::test]
    async fn test_update_theme_admin_session() {
        let mut mock_db = MockDatabaseOperations::new();
        let test_session = SessionModel::default();
        let mut admin_session = test_session.clone();
        admin_session.is_admin = true;
        let extension = Extension(admin_session.clone());
        let user_id = test_session.user_id.clone();

        mock_db
            .expect_update_field()
            .with(eq(user_id), eq("theme"), eq(0)) // Assuming 0 represents light theme
            .times(1)
            .returning(|_, _, _| Ok(()));

        let service = ClientService {
            db: Database::Mock(mock_db),
        };

        let request = Json(UpdateThemeRequest { theme: 0 });

        let response = update_theme(State(service), extension, request).await;

        let response = response.into_response();
        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_update_theme_toggle() {
        let mut mock_db = MockDatabaseOperations::new();
        let test_session = SessionModel::default();
        let extension = Extension(test_session.clone());
        let user_id = test_session.user_id.clone();

        // First update: light theme
        mock_db
            .expect_update_field()
            .with(eq(user_id.clone()), eq("theme"), eq(0))
            .times(1)
            .returning(|_, _, _| Ok(()));

        let service = ClientService {
            db: Database::Mock(mock_db),
        };

        let request = Json(UpdateThemeRequest { theme: 0 });

        let response = update_theme(State(service), extension, request).await;

        let response = response.into_response();
        assert_eq!(response.status(), StatusCode::OK);

        let mut mock_db = MockDatabaseOperations::new();
        let test_session = SessionModel::default();
        let extension = Extension(test_session.clone());
        let user_id = test_session.user_id.clone();

        // Second update: dark theme
        mock_db
            .expect_update_field()
            .with(eq(user_id.clone()), eq("theme"), eq(1))
            .times(1)
            .returning(|_, _, _| Ok(()));

        let service = ClientService {
            db: Database::Mock(mock_db),
        };

        let request = Json(UpdateThemeRequest { theme: 1 });

        let response = update_theme(State(service), extension, request).await;

        let response = response.into_response();
        assert_eq!(response.status(), StatusCode::OK);
    }
}
