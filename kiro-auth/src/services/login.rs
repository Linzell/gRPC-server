// services/login.rs
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

use kiro_api::google::protobuf::Timestamp;
use kiro_client::UserModel;
use tonic::{Request, Response, Status};

use crate::{
    utils::{ip::get_ip_from_md, password::valid_password},
    SessionStore,
};

/// Login service implementation
///
/// # Description
/// Authenticates a user and creates a new session
///
/// # Arguments
/// * `service` - The auth service instance
/// * `request` - Login request containing email and password
///
/// # Returns
/// * `Ok(Response)` - Response containing session token and expiry
/// * `Err(Status)` - Error status with description
///
/// # Errors
/// * `Status::invalid_argument` - Invalid password format
/// * `Status::not_found` - User not found
/// * `Status::permission_denied` - Invalid password
/// * `Status::internal` - Database or internal error
///
/// # Example
/// ```no_run
/// let request = Request::new(LoginRequest {
///     email: "user@example.com".to_string(),
///     password: "password123!".to_string()
/// });
/// let response = login(&service, request).await?;
/// let session = response.into_inner();
/// ```
pub async fn login(
    service: &AuthService, request: Request<LoginRequest>,
) -> Result<Response<Session>, Status> {
    // Extract IP address from request metadata
    let ip_address = get_ip_from_md(request.metadata()).unwrap_or_else(|| "unknown".to_string());

    let request = request.into_inner();

    // Validate password format
    if let Err(e) = valid_password(&request.password) {
        return Err(Status::invalid_argument(e.to_string()));
    }

    // Get user by email
    let user = UserModel::get_user_by_email(&service.db, request.email.clone())
        .await
        .map_err(|e| Status::not_found(format!("User not found: {}", e)))?;

    // Verify password
    let verified = SessionStore::verify_password(&service.db, request.password, user.password_hash)
        .await
        .map_err(|e| Status::internal(format!("Password verification error: {}", e)))?;

    if !verified {
        return Err(Status::permission_denied("Invalid password"));
    }

    // Create or get existing session
    let session = SessionStore::get_session_by_user_id(&service.db, user.id.clone(), ip_address)
        .await
        .map_err(|e| Status::internal(format!("Session creation failed: {}", e)))?;

    // Generate refresh token
    let refresh_token = SessionStore::generate_refresh_token(session.user_id)
        .await
        .map_err(|e| Status::internal(format!("Refresh token generation failed: {}", e)))?;

    let expire_date: Option<Timestamp> = Some(Timestamp {
        seconds: (chrono::Utc::now() + chrono::Duration::days(2)).timestamp(),
        nanos: 0,
    });

    Ok(Response::new(Session {
        token: refresh_token.1,
        expire_date,
    }))
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::{CreateSessionModel, SessionModel};
    use chrono::Utc;
    use kiro_client::{
        Language, NotificationSettings, PrivacySettings, SecuritySettings, Theme, UserSettings,
    };
    use kiro_database::{db_bridge::MockDatabaseOperations, DbDateTime, DbId};
    use mockall::predicate::{always, eq};

    fn create_test_user() -> UserModel {
        UserModel {
            id: DbId::from(("users", "123")),
            customer_id: Some("cust_123".to_string()),
            email: "test@example.com".to_string(),
            password_hash: "hashed_password".to_string(),
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

        mock_db
            .expect_read_by_field::<UserModel>()
            .with(eq("users"), eq("email"), eq("test@example.com"), eq(None))
            .times(1)
            .returning(|_, _, _, _| Ok(vec![create_test_user()]));

        mock_db
            .expect_query::<bool>()
            .times(1)
            .returning(|_, _| Ok(vec![true]));

        mock_db
            .expect_read_by_field_thing::<SessionModel>()
            .with(
                eq("sessions"),
                eq("user_id"),
                eq(DbId::from(("users", "1"))),
                eq(None),
            )
            .times(1)
            .returning(|_, _, _, _| Ok(vec![]));

        mock_db
            .expect_query::<String>()
            .with(eq("RETURN crypto::sha256(rand::string(50));"), eq(None))
            .times(1)
            .returning(|_, _| Ok(vec!["session_token".to_string()]));

        let session = create_test_session();
        mock_db
            .expect_create::<SessionModel, SessionModel>()
            .with(eq("sessions"), always())
            .times(1)
            .return_once(move |_, _| Ok(vec![session]));

        let service = AuthService {
            db: Database::Mock(mock_db),
        };

        let request = Request::new(LoginRequest {
            email: "test@example.com".to_string(),
            password: "Password123!".to_string(),
        });

        let response = login(&service, request).await.unwrap().into_inner();
        assert_eq!(response.token, "session_token");
        assert!(!response.expire_date.is_none());
    }

    #[tokio::test]
    async fn test_login_invalid_password() {
        let mut mock_db = MockDatabaseOperations::new();

        mock_db
            .expect_read_by_field::<UserModel>()
            .withf(|collection, field, value, _| {
                collection == "users" && field == "email" && value == "test@example.com"
            })
            .times(1)
            .returning(|_, _, _, _| Ok(vec![create_test_user()]));

        mock_db
            .expect_query::<bool>()
            .times(1)
            .returning(|_, _| Ok(vec![false]));

        let service = AuthService {
            db: Database::Mock(mock_db),
        };

        let request = Request::new(LoginRequest {
            email: "test@example.com".to_string(),
            password: "Password123!".to_string(),
        });

        let error = login(&service, request).await.unwrap_err();
        assert_eq!(error.code(), tonic::Code::PermissionDenied);
        assert_eq!(error.message(), "Invalid password");
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

        let request = Request::new(LoginRequest {
            email: "nonexistent@example.com".to_string(),
            password: "Password123!".to_string(),
        });

        let error = login(&service, request).await.unwrap_err();
        assert_eq!(error.code(), tonic::Code::NotFound);
        assert!(error.message().contains("User not found"));
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

        let request = Request::new(LoginRequest {
            email: "test@example.com".to_string(),
            password: "Password123!".to_string(),
        });

        let error = login(&service, request).await.unwrap_err();
        assert_eq!(error.code(), tonic::Code::NotFound);
    }

    #[tokio::test]
    async fn test_login_invalid_password_format() {
        let mock_db = MockDatabaseOperations::new();
        let service = AuthService {
            db: Database::Mock(mock_db),
        };

        let request = Request::new(LoginRequest {
            email: "test@example.com".to_string(),
            password: "short".to_string(),
        });

        let error = login(&service, request).await.unwrap_err();
        assert_eq!(error.code(), tonic::Code::InvalidArgument);
        assert!(error.message().contains("Password too short"));
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
            .expect_query::<bool>()
            .times(1)
            .returning(|_, _| Ok(vec![true]));

        mock_db
            .expect_read_by_field_thing::<SessionModel>()
            .with(
                eq("sessions"),
                eq("user_id"),
                eq(DbId::from(("users", "1"))),
                eq(None),
            )
            .times(1)
            .returning(|_, _, _, _| Ok(vec![]));

        mock_db
            .expect_query::<String>()
            .with(eq("RETURN crypto::sha256(rand::string(50));"), eq(None))
            .times(1)
            .returning(|_, _| Ok(vec!["session_token".to_string()]));

        mock_db
            .expect_create::<CreateSessionModel, SessionModel>()
            .with(eq("sessions"), always())
            .times(1)
            .return_once(|_, _| {
                Err(kiro_database::DatabaseError::Internal(
                    "Session creation failed".to_string(),
                ))
            });

        let service = AuthService {
            db: Database::Mock(mock_db),
        };

        let request = Request::new(LoginRequest {
            email: "test@example.com".to_string(),
            password: "Password123!".to_string(),
        });

        let error = login(&service, request).await.unwrap_err();
        assert_eq!(error.code(), tonic::Code::Internal);
        assert!(error.message().contains("Session creation failed"));
    }
}
