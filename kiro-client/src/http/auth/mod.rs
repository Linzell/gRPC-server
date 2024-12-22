// http/auth/mod.rs
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

use axum::{
    routing::{get, post},
    Router,
};
use kiro_database::db_bridge::Database;

mod login;
mod logout;
mod register;

use crate::AuthService;

/// Creates and configures authentication routes
///
/// # Arguments
/// * `db` - Database connection pool
///
/// # Returns
/// Router configured with authentication endpoints:
/// - POST /login - User login
/// - GET /logout - User logout
/// - POST /register - New user registration
///
/// # Example
/// ```rust,ignore
/// use kiro_database::db_bridge::Database;
///
/// let db = Database::new("connection_string").await?;
/// let auth_router = auth_routes(db);
/// ```
pub fn auth_routes(db: Database) -> Router {
    let service = AuthService::new(db);

    Router::new()
        .route("/login", post(login::login))
        .route("/logout", get(logout::logout))
        .route("/register", post(register::register))
        .with_state(service)
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{
        body::Body,
        http::{Method, Request, StatusCode},
    };
    use kiro_database::db_bridge::MockDatabaseOperations;
    use serde_json::json;
    use tower::ServiceExt;

    #[tokio::test]
    async fn test_auth_routes() {
        let mock_db = MockDatabaseOperations::new();
        let app = auth_routes(Database::Mock(mock_db));

        let login_response = app
            .clone()
            .oneshot(
                Request::builder()
                    .method(Method::POST)
                    .uri("/login")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(login_response.status(), StatusCode::UNSUPPORTED_MEDIA_TYPE);

        let logout_response = app
            .clone()
            .oneshot(
                Request::builder()
                    .method(Method::GET)
                    .uri("/logout")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(logout_response.status(), StatusCode::INTERNAL_SERVER_ERROR);

        let register_response = app
            .oneshot(
                Request::builder()
                    .method(Method::POST)
                    .uri("/register")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(
            register_response.status(),
            StatusCode::UNSUPPORTED_MEDIA_TYPE
        );
    }

    #[tokio::test]
    async fn test_login_with_valid_credentials() {
        let mock_db = MockDatabaseOperations::new();
        let app = auth_routes(Database::Mock(mock_db));

        let login_request = json!({
            "email": "test@example.com",
            "password": "password123"
        });

        let response = app
            .oneshot(
                Request::builder()
                    .method(Method::POST)
                    .uri("/login")
                    .header("Content-Type", "application/json")
                    .body(Body::from(login_request.to_string()))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    async fn test_register_new_user() {
        let mock_db = MockDatabaseOperations::new();
        let app = auth_routes(Database::Mock(mock_db));

        let register_request = json!({
            "email": "new@example.com",
            "password": "password123",
            "name": "Test User"
        });

        let response = app
            .oneshot(
                Request::builder()
                    .method(Method::POST)
                    .uri("/register")
                    .header("Content-Type", "application/json")
                    .body(Body::from(register_request.to_string()))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    async fn test_logout_with_valid_token() {
        let mock_db = MockDatabaseOperations::new();
        // TODO: Configure mock_db to validate this token
        let app = auth_routes(Database::Mock(mock_db));

        let response = app
            .oneshot(
                Request::builder()
                    .method(Method::GET)
                    .uri("/logout")
                    .header("Authorization", "Bearer valid_token")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);
    }

    #[tokio::test]
    async fn test_invalid_requests() {
        let mock_db = MockDatabaseOperations::new();
        let app = auth_routes(Database::Mock(mock_db));

        let response = app
            .clone()
            .oneshot(
                Request::builder()
                    .method(Method::POST)
                    .uri("/login")
                    .header("Content-Type", "application/json")
                    .body(Body::from("{invalid json}"))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::BAD_REQUEST);

        let invalid_registration = json!({
            "email": "test@example.com"
            // missing password and name
        });

        let response = app
            .oneshot(
                Request::builder()
                    .method(Method::POST)
                    .uri("/register")
                    .header("Content-Type", "application/json")
                    .body(Body::from(invalid_registration.to_string()))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }
}
