// services/http/register.rs
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
use kiro_database::db_bridge::DatabaseOperations;

use crate::{
    utils::{ip::get_ip_from_headers, password::valid_password},
    CreateUserModel, SessionModel, UserModel,
};

/// Register service implementation
///
/// # Description
/// Registers a new user with the system
///
/// # Arguments
/// * `service` - The authentication service instance
/// * `request` - The registration request containing email and password
///
/// # Returns
/// * `Ok(Session)` - The session token and expiry date
/// * `Err(Status)` - Appropriate error status on failure
///
/// # Errors
/// * `INVALID_ARGUMENT` - Invalid email or password
/// * `INTERNAL_SERVER_ERROR` - Database error
/// * `BAD_REQUEST` - Email already in use
///
/// # Example
/// ```no_run
/// let request = Request::new(LoginRequest {
///     email: "user@example.com".to_string(),
///     password: "password123!".to_string()
/// });
/// let response = register(service, request).await?;
/// let session = response.into_inner();
/// ```
pub async fn register(
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

    // Check if email is already in use
    match UserModel::check_email(&service.db, request.email.clone()).await {
        Ok(true) => {}
        Ok(false) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(serde_json::json!({ "error": "Email already in use" })),
            )
                .into_response()
        }
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({ "error": e.to_string() })),
            )
                .into_response()
        }
    };

    let password_hash = match SessionModel::create_password_hash(request.password.clone()).await {
        Ok(hash) => hash,
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({ "error": e.to_string() })),
            )
                .into_response()
        }
    };

    // Create new user
    let user = match service
        .db
        .create::<CreateUserModel, UserModel>(
            "users",
            CreateUserModel {
                email: request.email.clone(),
                password_hash,
            },
        )
        .await
    {
        Ok(user) => user.into_iter().next().unwrap(),
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({ "error": e.to_string() })),
            )
                .into_response()
        }
    };

    // Create session
    let session =
        match SessionModel::create_session(&service.db, user.id.clone(), false, Some(ip_address))
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
    async fn test_register_success() {
        let mut mock_db = MockDatabaseOperations::new();
        let user = create_test_user();

        mock_db
            .expect_read_by_field::<UserModel>()
            .with(eq("users"), eq("email"), eq("test@example.com"), eq(None))
            .times(1)
            .returning(|_, _, _, _| Ok(vec![]));

        mock_db
            .expect_create::<CreateUserModel, UserModel>()
            .with(eq("users"), always())
            .times(1)
            .returning(move |_, _| Ok(vec![user.clone()]));

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
        let response = register(State(service), headers, request).await;

        let response = response.into_response();
        let body_bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let session: Session = serde_json::from_slice(&body_bytes).unwrap();

        assert!(!session.token.is_empty());
        assert!(session.expire_date.is_some());
    }

    #[tokio::test]
    async fn test_register_invalid_password() {
        let mock_db = MockDatabaseOperations::new();

        let service = AuthService {
            db: Database::Mock(mock_db),
        };

        let request = Json(AuthRequest {
            email: "test@example.com".to_string(),
            password: "invalid".to_string(),
        });

        let headers = HeaderMap::new();
        let response = register(State(service), headers, request).await;

        let response = response.into_response();
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);

        let body_bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let error: serde_json::Value = serde_json::from_slice(&body_bytes).unwrap();
        assert_eq!(
            error["error"],
            "Password too short. Minimum size: 8 characters"
        );
    }

    #[tokio::test]
    async fn test_register_email_in_use() {
        let mut mock_db = MockDatabaseOperations::new();
        let user = create_test_user();

        mock_db
            .expect_read_by_field::<UserModel>()
            .with(eq("users"), eq("email"), eq("test@example.com"), eq(None))
            .times(1)
            .returning(move |_, _, _, _| Ok(vec![user.clone()]));

        let service = AuthService {
            db: Database::Mock(mock_db),
        };

        let request = Json(AuthRequest {
            email: "test@example.com".to_string(),
            password: "Password123!".to_string(),
        });

        let headers = HeaderMap::new();
        let response = register(State(service), headers, request).await;

        let response = response.into_response();
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);

        let body_bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let error: serde_json::Value = serde_json::from_slice(&body_bytes).unwrap();
        assert_eq!(error["error"], "Email already in use");
    }

    #[tokio::test]
    async fn test_register_user_creation_failure() {
        let mut mock_db = MockDatabaseOperations::new();

        mock_db
            .expect_read_by_field::<UserModel>()
            .with(eq("users"), eq("email"), eq("test@example.com"), eq(None))
            .times(1)
            .returning(|_, _, _, _| Ok(vec![]));

        mock_db
            .expect_create::<CreateUserModel, UserModel>()
            .with(eq("users"), always())
            .times(1)
            .returning(|_, _| Err(DatabaseError::Internal("Failed to create user".to_string())));

        let service = AuthService {
            db: Database::Mock(mock_db),
        };

        let request = Json(AuthRequest {
            email: "test@example.com".to_string(),
            password: "Password123!".to_string(),
        });

        let headers = HeaderMap::new();
        let response = register(State(service), headers, request).await;

        let response = response.into_response();
        assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);

        let body_bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let error: serde_json::Value = serde_json::from_slice(&body_bytes).unwrap();
        assert_eq!(error["error"], "Internal error: Failed to create user");
    }

    #[tokio::test]
    async fn test_register_session_creation_failure() {
        let mut mock_db = MockDatabaseOperations::new();
        let user = create_test_user();

        mock_db
            .expect_read_by_field::<UserModel>()
            .with(eq("users"), eq("email"), eq("test@example.com"), eq(None))
            .times(1)
            .returning(|_, _, _, _| Ok(vec![]));

        mock_db
            .expect_create::<CreateUserModel, UserModel>()
            .with(eq("users"), always())
            .times(1)
            .returning(move |_, _| Ok(vec![user.clone()]));

        mock_db
            .expect_create::<CreateSessionModel, SessionModel>()
            .with(eq("sessions"), always())
            .times(1)
            .returning(|_, _| {
                Err(DatabaseError::Internal(
                    "Failed to create session".to_string(),
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
        let response = register(State(service), headers, request).await;

        let response = response.into_response();
        assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);

        let body_bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let error: serde_json::Value = serde_json::from_slice(&body_bytes).unwrap();
        assert_eq!(error["error"], "Internal error: Failed to create session");
    }
}
