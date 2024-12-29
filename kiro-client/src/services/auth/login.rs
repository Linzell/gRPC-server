// services/auth/login.rs
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
use tonic::{Request, Response, Status};

use crate::{
    models::UserModel,
    utils::{ip::get_ip_from_md, password::valid_password},
    SessionModel,
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
/// ```rust,no_run
/// use tonic::{Request, Response, Status};
/// use kiro_api::auth::v1::{auth_service_server::AuthService, AuthRequest};
/// use kiro_database::db_bridge::{Database, MockDatabaseOperations};
///
/// // Mock database
/// let mock_db = MockDatabaseOperations::new();
///
/// // Mock service
/// let service = kiro_client::AuthService {
///     db: Database::Mock(mock_db),
/// };
///
/// // Login request
/// let request = Request::new(AuthRequest {
///     email: "user@example.com".to_string(),
///     password: "password123!".to_string()
/// });
///
/// // Async block to allow `await`
/// tokio::runtime::Runtime::new().unwrap().block_on(async {
///     AuthService::login(&service, request).await;
///
///     println!("Login successful");
/// });
/// ```
pub async fn login(
    service: &AuthService, request: Request<AuthRequest>,
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
    let verified = SessionModel::verify_password(request.password, user.password_hash)
        .await
        .map_err(|e| Status::internal(format!("Password verification error: {}", e)))?;

    if !verified {
        return Err(Status::permission_denied("Invalid password"));
    }

    // Create or get existing session
    let session = SessionModel::get_session_by_user_id(&service.db, user.id.clone(), ip_address)
        .await
        .map_err(|e| Status::internal(format!("Session creation failed: {}", e)))?;

    // Generate refresh token
    let refresh_token = SessionModel::generate_refresh_token(session.user_id)
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
    use kiro_database::{db_bridge::MockDatabaseOperations, DatabaseError, DbId};
    use mockall::predicate::{always, eq};

    #[tokio::test]
    async fn test_login_success() {
        let mut mock_db = MockDatabaseOperations::new();
        let test_user = UserModel::default();

        mock_db
            .expect_read_by_field::<UserModel>()
            .with(eq("users"), eq("email"), eq("test@example.com"), eq(None))
            .times(1)
            .returning(move |_, _, _, _| Ok(vec![test_user.clone()]));

        mock_db
            .expect_read_by_field_thing::<SessionModel>()
            .with(eq("sessions"), eq("user_id"), eq(DbId::default()), eq(None))
            .times(1)
            .returning(|_, _, _, _| Ok(vec![]));

        let session = SessionModel::default();
        mock_db
            .expect_create::<CreateSessionModel, SessionModel>()
            .with(eq("sessions"), always())
            .times(1)
            .return_once(move |_, _| Ok(vec![session]));

        let service = AuthService {
            db: Database::Mock(mock_db),
        };

        let request = Request::new(AuthRequest {
            email: "test@example.com".to_string(),
            password: "Password123!".to_string(),
        });

        let response = login(&service, request).await.unwrap().into_inner();

        assert!(!response.token.is_empty());
        assert!(response.expire_date.is_some());
    }

    #[tokio::test]
    async fn test_login_invalid_password() {
        let mut mock_db = MockDatabaseOperations::new();

        mock_db
            .expect_read_by_field::<UserModel>()
            .with(eq("users"), eq("email"), eq("test@example.com"), eq(None))
            .times(1)
            .returning(|_, _, _, _| Ok(vec![UserModel::default()]));

        let service = AuthService {
            db: Database::Mock(mock_db),
        };

        let request = Request::new(AuthRequest {
            email: "test@example.com".to_string(),
            password: "WrongPassword123!".to_string(),
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

        let request = Request::new(AuthRequest {
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

        let request = Request::new(AuthRequest {
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

        let request = Request::new(AuthRequest {
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
            .returning(|_, _, _, _| Ok(vec![UserModel::default()]));

        mock_db
            .expect_read_by_field_thing::<SessionModel>()
            .with(eq("sessions"), eq("user_id"), eq(DbId::default()), eq(None))
            .times(1)
            .returning(|_, _, _, _| {
                Err(DatabaseError::Internal(
                    "Session creation failed".to_string(),
                ))
            });

        let service = AuthService {
            db: Database::Mock(mock_db),
        };

        let request = Request::new(AuthRequest {
            email: "test@example.com".to_string(),
            password: "Password123!".to_string(),
        });

        let error = login(&service, request).await.unwrap_err();
        assert_eq!(error.code(), tonic::Code::Internal);
        assert!(error.message().contains("Session creation failed"));
    }
}
