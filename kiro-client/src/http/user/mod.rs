// http/user/mod.rs
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
    routing::{delete, get, post},
    Router,
};
use kiro_database::db_bridge::Database;

pub mod delete_user;
pub mod disable_user;
pub mod read_user;
#[cfg(feature = "mailer")]
pub mod send_email_to_change_email;
#[cfg(feature = "mailer")]
pub mod send_email_to_change_password;
pub mod update_email;
pub mod update_language;
pub mod update_notifications;
pub mod update_password;
pub mod update_privacy;
pub mod update_security;
pub mod update_theme;
#[cfg(feature = "storage")]
pub mod upload_avatar;

use crate::ClientService;

/// Creates and configures user routes
///
/// # Arguments
/// * `db` - Database connection pool
///
/// # Returns
/// Router configured with user endpoints:
/// - DELETE /delete_user - Delete user
/// - DELETE /disable_user - Disable user
/// - GET /read_user - Read user
/// - POST /update_email - Update email
/// - POST /update_language - Update language
/// - POST /update_notifications - Update notifications
/// - POST /update_password - Update password
/// - POST /update_privacy - Update privacy
/// - POST /update_security - Update security
/// - POST /update_theme - Update theme
/// - GET /send_email_to_change_email - Send email to change email
/// - GET /send_email_to_change_password - Send email to change password
/// - POST /upload_avatar - Upload avatar
///
/// # Example
/// ```rust,no_run
/// use kiro_client::user_routes;
/// use kiro_database::db_bridge::{Database, MockDatabaseOperations};
///
/// let mock_db = MockDatabaseOperations::new();
///
/// user_routes(Database::Mock(mock_db));
///
/// println!("User routes created");
/// ```
pub fn user_routes(db: Database) -> Router {
    let service = ClientService::new(db);

    let mut router = Router::new();

    router = router
        .route("/delete_user", delete(delete_user::delete_user))
        .route("/disable_user", delete(disable_user::disable_user))
        .route("/read_user", get(read_user::read_user))
        .route("/update_email", post(update_email::update_email))
        .route("/update_language", post(update_language::update_language))
        .route(
            "/update_notifications",
            post(update_notifications::update_notifications),
        )
        .route("/update_password", post(update_password::update_password))
        .route("/update_privacy", post(update_privacy::update_privacy))
        .route("/update_security", post(update_security::update_security))
        .route("/update_theme", post(update_theme::update_theme));

    #[cfg(feature = "mailer")]
    {
        router = router
            .route(
                "/send_email_to_change_email",
                get(send_email_to_change_email::send_email_to_change_email),
            )
            .route(
                "/send_email_to_change_password",
                get(send_email_to_change_password::send_email_to_change_password),
            );
    }

    #[cfg(feature = "storage")]
    {
        router = router.route("/upload_avatar", post(upload_avatar::upload_avatar));
    }

    router.with_state(service)
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{
        body::Body,
        http::{Method, Request, StatusCode},
    };
    use kiro_database::db_bridge::MockDatabaseOperations;
    use tower::ServiceExt;

    #[tokio::test]
    async fn test_user_routes() {
        let mock_db = MockDatabaseOperations::new();
        let app = user_routes(Database::Mock(mock_db));

        let delete_user_response = app
            .clone()
            .oneshot(
                Request::builder()
                    .method(Method::DELETE)
                    .uri("/delete_user")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(
            delete_user_response.status(),
            StatusCode::INTERNAL_SERVER_ERROR
        );

        let disable_user_response = app
            .clone()
            .oneshot(
                Request::builder()
                    .method(Method::DELETE)
                    .uri("/disable_user")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(
            disable_user_response.status(),
            StatusCode::INTERNAL_SERVER_ERROR
        );

        let read_user_response = app
            .clone()
            .oneshot(
                Request::builder()
                    .method(Method::GET)
                    .uri("/read_user")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(
            read_user_response.status(),
            StatusCode::INTERNAL_SERVER_ERROR
        );

        let update_email_response = app
            .clone()
            .oneshot(
                Request::builder()
                    .method(Method::POST)
                    .uri("/update_email")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(
            update_email_response.status(),
            StatusCode::INTERNAL_SERVER_ERROR
        );

        let update_language_response = app
            .clone()
            .oneshot(
                Request::builder()
                    .method(Method::POST)
                    .uri("/update_language")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(
            update_language_response.status(),
            StatusCode::INTERNAL_SERVER_ERROR
        );

        let update_notifications_response = app
            .clone()
            .oneshot(
                Request::builder()
                    .method(Method::POST)
                    .uri("/update_notifications")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(
            update_notifications_response.status(),
            StatusCode::INTERNAL_SERVER_ERROR
        );

        let update_password_response = app
            .clone()
            .oneshot(
                Request::builder()
                    .method(Method::POST)
                    .uri("/update_password")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(
            update_password_response.status(),
            StatusCode::INTERNAL_SERVER_ERROR
        );

        let update_privacy_response = app
            .clone()
            .oneshot(
                Request::builder()
                    .method(Method::POST)
                    .uri("/update_privacy")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(
            update_privacy_response.status(),
            StatusCode::INTERNAL_SERVER_ERROR
        );

        let update_security_response = app
            .clone()
            .oneshot(
                Request::builder()
                    .method(Method::POST)
                    .uri("/update_security")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(
            update_security_response.status(),
            StatusCode::INTERNAL_SERVER_ERROR
        );

        let update_theme_response = app
            .clone()
            .oneshot(
                Request::builder()
                    .method(Method::POST)
                    .uri("/update_theme")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(
            update_theme_response.status(),
            StatusCode::INTERNAL_SERVER_ERROR
        );

        #[cfg(feature = "mailer")]
        {
            let send_email_to_change_email_response = app
                .clone()
                .oneshot(
                    Request::builder()
                        .method(Method::GET)
                        .uri("/send_email_to_change_email")
                        .body(Body::empty())
                        .unwrap(),
                )
                .await
                .unwrap();
            assert_eq!(
                send_email_to_change_email_response.status(),
                StatusCode::INTERNAL_SERVER_ERROR
            );

            let send_email_to_change_password_response = app
                .clone()
                .oneshot(
                    Request::builder()
                        .method(Method::GET)
                        .uri("/send_email_to_change_password")
                        .body(Body::empty())
                        .unwrap(),
                )
                .await
                .unwrap();
            assert_eq!(
                send_email_to_change_password_response.status(),
                StatusCode::INTERNAL_SERVER_ERROR
            );
        }

        #[cfg(feature = "storage")]
        {
            let upload_avatar_response = app
                .oneshot(
                    Request::builder()
                        .method(Method::POST)
                        .uri("/upload_avatar")
                        .body(Body::empty())
                        .unwrap(),
                )
                .await
                .unwrap();
            assert_eq!(
                upload_avatar_response.status(),
                StatusCode::INTERNAL_SERVER_ERROR
            );
        }
    }
}
