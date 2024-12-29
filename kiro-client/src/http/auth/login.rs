// http/auth/login.rs
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

use axum::{extract::State, http::StatusCode, response::IntoResponse, Json};
use http::HeaderMap;
use kiro_api::{
    auth::v1::{AuthRequest, Session},
    google::protobuf::Timestamp,
};

use crate::{
    utils::{ip::get_ip_from_headers, password::valid_password},
    SessionModel, UserModel,
};

/// Login service implementation
///
/// # Description
/// Authenticates a user and creates a new session
///
/// # Arguments
/// * `service` - The authentication service instance
/// * `headers` - HTTP headers containing IP address and other metadata
/// * `request` - The login request containing email and password
///
/// # Returns
/// * HTTP response with either:
///   * `200 OK` with Session containing token and expiry
///   * Error status code with message
///
/// # Errors
/// * `400 BAD REQUEST` - Invalid password format
/// * `401 UNAUTHORIZED` - Invalid password
/// * `404 NOT FOUND` - User not found
/// * `500 INTERNAL SERVER ERROR` - Database or server error
///
/// # Example
/// ```rust,ignore
/// use axum::{Json, http::HeaderMap};
/// use kiro_api::auth::v1::AuthRequest;
///
/// let request = Json(AuthRequest {
///     email: "user@example.com".to_string(),
///     password: "password123!".to_string()
/// });
/// let headers = HeaderMap::new();
/// let response = login(State(service), headers, request).await;
/// ```
#[utoipa::path(
    post,
    path = "/auth/login",
    tag = "auth",
    params(
        AuthRequest
    ),
    responses(
        (status = 200, description = "Session created", body = Session),
        (status = 400, description = "Invalid password format", body = String),
        (status = 401, description = "Invalid password", body = String),
        (status = 404, description = "User not found", body = String),
        (status = 409, description = "User already exists", body = String),
        (status = 500, description = "Internal server error", body = String)

    )
)]
pub async fn login(
    State(service): State<AuthService>, headers: HeaderMap, Json(request): Json<AuthRequest>,
) -> impl IntoResponse {
    // Extract IP address from request metadata
    let ip_address = get_ip_from_headers(&headers).unwrap_or_else(|| "unknown".to_string());

    if let Err(e) = valid_password(&request.password) {
        return (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({ "error": e.to_string() })),
        )
            .into_response();
    }

    // Check if the user exists
    let user = match UserModel::get_user_by_email(&service.db, request.email).await {
        Ok(user) => user,
        Err(e) => {
            return (
                StatusCode::NOT_FOUND,
                Json(serde_json::json!({ "error": e.to_string() })),
            )
                .into_response()
        }
    };

    // Verify password
    let verified = match SessionModel::verify_password(request.password, user.password_hash).await {
        Ok(verified) => verified,
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({ "error": e.to_string() })),
            )
                .into_response()
        }
    };

    if !verified {
        return (
            StatusCode::UNAUTHORIZED,
            Json(serde_json::json!({ "error": "Invalid password" })),
        )
            .into_response();
    }

    // Create or get existing session
    let session = match SessionModel::get_session_by_user_id(
        &service.db,
        user.id.clone(),
        ip_address,
    )
    .await
    {
        Ok(session) => session,
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({ "error": e.to_string() })),
            )
                .into_response()
        }
    };

    // Generate refresh token
    let refresh_token = match SessionModel::generate_refresh_token(session.user_id).await {
        Ok(token) => token,
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({ "error": e.to_string() })),
            )
                .into_response()
        }
    };

    let expire_date: Option<Timestamp> = Some(Timestamp {
        seconds: (chrono::Utc::now() + chrono::Duration::days(2)).timestamp(),
        nanos: 0,
    });

    // Return session
    (
        StatusCode::OK,
        Json(Session {
            token: refresh_token.1,
            expire_date,
        }),
    )
        .into_response()
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::{
        CreateSessionModel, Language, NotificationSettings, PrivacySettings, SecuritySettings,
        SessionModel, Theme, UserSettings,
    };
    use argon2::{password_hash::SaltString, Argon2, PasswordHasher};
    use chrono::Utc;
    use kiro_database::{db_bridge::MockDatabaseOperations, DatabaseError, DbDateTime, DbId};
    use mockall::predicate::{always, eq};
    use rand_core::OsRng;

    fn create_test_user() -> UserModel {
        // Create a proper password hash for testing
        let salt = SaltString::generate(&mut OsRng);
        let argon2 = Argon2::default();
        let password_hash = argon2
            .hash_password("Password123!".as_bytes(), &salt)
            .unwrap()
            .to_string();

        UserModel {
            id: DbId::from(("users", "123")),
            customer_id: Some("cust_123".to_string()),
            email: "test@example.com".to_string(),
            password_hash,
            avatar: Some("avatar.jpg".to_string()),
            settings: UserSettings {
                language: Some(Language::English),
                theme: Some(Theme::Dark),
                notifications: NotificationSettings {
                    email: true,
                    push: true,
                    sms: false,
                },
                privacy: PrivacySettings {
                    data_collection: true,
                    location: false,
                },
                security: SecuritySettings {
                    two_factor: true,
                    qr_code: "qr_code".to_string(),
                    magic_link: true,
                },
            },
            groups: vec![DbId::from(("groups", "123"))],
            created_at: DbDateTime::from(Utc::now()),
            updated_at: DbDateTime::from(Utc::now()),
            activated: true,
            is_admin: false,
        }
    }

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
    async fn test_login_success() {
        let mut mock_db = MockDatabaseOperations::new();
        let test_user = create_test_user();

        mock_db
            .expect_read_by_field::<UserModel>()
            .with(eq("users"), eq("email"), eq("test@example.com"), eq(None))
            .times(1)
            .returning(move |_, _, _, _| Ok(vec![test_user.clone()]));

        mock_db
            .expect_read_by_field_thing::<SessionModel>()
            .with(
                eq("sessions"),
                eq("user_id"),
                eq(DbId::from(("users", "123"))),
                eq(None),
            )
            .times(1)
            .returning(|_, _, _, _| Ok(vec![]));

        let session = create_test_session();
        mock_db
            .expect_create::<CreateSessionModel, SessionModel>()
            .with(eq("sessions"), always())
            .times(1)
            .return_once(move |_, _| Ok(vec![session]));

        let service = AuthService {
            db: Database::Mock(mock_db),
        };

        let request = Json(AuthRequest {
            email: "test@example.com".to_string(),
            password: "Password123!".to_string(),
        });

        let headers = HeaderMap::new();
        let response = login(State(service), headers, request).await;

        let response = response.into_response();
        let body_bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let session: Session = serde_json::from_slice(&body_bytes).unwrap();

        assert!(!session.token.is_empty());
        assert!(session.expire_date.is_some());
    }

    #[tokio::test]
    async fn test_login_invalid_password() {
        let mut mock_db = MockDatabaseOperations::new();

        mock_db
            .expect_read_by_field::<UserModel>()
            .with(eq("users"), eq("email"), eq("test@example.com"), eq(None))
            .times(1)
            .returning(|_, _, _, _| Ok(vec![create_test_user()]));

        let service = AuthService {
            db: Database::Mock(mock_db),
        };

        let request = Json(AuthRequest {
            email: "test@example.com".to_string(),
            password: "WrongPassword123!".to_string(),
        });

        let headers = HeaderMap::new();
        let response = login(State(service), headers, request).await;

        let response = response.into_response();
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);

        let body_bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let error: serde_json::Value = serde_json::from_slice(&body_bytes).unwrap();

        assert_eq!(error["error"], "Invalid password");
    }

    #[tokio::test]
    async fn test_login_user_not_found() {
        let mut mock_db = MockDatabaseOperations::new();

        mock_db
            .expect_read_by_field::<UserModel>()
            .withf(|collection, field, value, _| {
                collection == "users" && field == "email" && value == "nonexistent@example.com"
            })
            .times(1)
            .returning(|_, _, _, _| Ok(vec![]));

        let service = AuthService {
            db: Database::Mock(mock_db),
        };

        let request = Json(AuthRequest {
            email: "nonexistent@example.com".to_string(),
            password: "Password123!".to_string(),
        });

        let headers = HeaderMap::new();
        let response = login(State(service), headers, request).await;

        let response = response.into_response();
        assert_eq!(response.status(), StatusCode::NOT_FOUND);

        let body_bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let error: serde_json::Value = serde_json::from_slice(&body_bytes).unwrap();

        assert_eq!(
            error["error"],
            "Database Record that was just checked doesn't exist?"
        );
    }

    #[tokio::test]
    async fn test_login_db_error() {
        let mut mock_db = MockDatabaseOperations::new();

        mock_db
            .expect_read_by_field::<UserModel>()
            .withf(|collection, field, value, _| {
                collection == "users" && field == "email" && value == "test@example.com"
            })
            .times(1)
            .returning(|_, _, _, _| Err(kiro_database::DatabaseError::DBOptionNone));

        let service = AuthService {
            db: Database::Mock(mock_db),
        };

        let request = Json(AuthRequest {
            email: "test@example.com".to_string(),
            password: "Password123!".to_string(),
        });

        let headers = HeaderMap::new();
        let response = login(State(service), headers, request).await;

        let response = response.into_response();
        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn test_login_invalid_password_format() {
        let mock_db = MockDatabaseOperations::new();
        let service = AuthService {
            db: Database::Mock(mock_db),
        };

        let request = Json(AuthRequest {
            email: "test@example.com".to_string(),
            password: "short".to_string(),
        });

        let headers = HeaderMap::new();
        let response = login(State(service), headers, request).await;

        let response = response.into_response();
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);

        let body_bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let error: serde_json::Value = serde_json::from_slice(&body_bytes).unwrap();

        assert!(error["error"]
            .as_str()
            .unwrap()
            .contains("Password too short"));
    }
    #[tokio::test]
    async fn test_login_session_creation_error() {
        let mut mock_db = MockDatabaseOperations::new();

        mock_db
            .expect_read_by_field::<UserModel>()
            .with(eq("users"), eq("email"), eq("test@example.com"), eq(None))
            .times(1)
            .returning(|_, _, _, _| Ok(vec![create_test_user()]));

        mock_db
            .expect_read_by_field_thing::<SessionModel>()
            .with(
                eq("sessions"),
                eq("user_id"),
                eq(DbId::from(("users", "123"))),
                eq(None),
            )
            .times(1)
            .returning(|_, _, _, _| {
                Err(DatabaseError::Internal(
                    "Session creation failed".to_string(),
                ))
            });

        let service = AuthService {
            db: Database::Mock(mock_db),
        };

        let request = Json(AuthRequest {
            email: "test@example.com".to_string(),
            password: "Password123!".to_string(),
        });

        let headers = HeaderMap::new();
        let response = login(State(service), headers, request).await;

        let response = response.into_response();
        assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);

        let body_bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let error: serde_json::Value = serde_json::from_slice(&body_bytes).unwrap();

        assert_eq!(error["error"], "Internal error: Session creation failed");
    }
}
