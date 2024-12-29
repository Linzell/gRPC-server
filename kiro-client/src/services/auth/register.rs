// services/auth/register.rs
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
use kiro_database::db_bridge::DatabaseOperations;
use tonic::{Request, Response, Status};

use crate::{
    utils::{ip::get_ip_from_md, password::valid_password},
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
/// * `Status::InvalidArgument` - Invalid password format
/// * `Status::Internal` - Database error
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
/// // Register request
/// let request = Request::new(AuthRequest {
///     email: "user@example.com".to_string(),
///     password: "password123!".to_string()
/// });
///
/// // Async block to allow `await`
/// tokio::runtime::Runtime::new().unwrap().block_on(async {
///     AuthService::register(&service, request).await;
///
///     println!("Login successful");
/// });
/// ```
pub async fn register(
    service: &AuthService, request: Request<AuthRequest>,
) -> Result<Response<Session>, Status> {
    // Extract IP address from request metadata
    let ip_address = get_ip_from_md(request.metadata()).unwrap_or_else(|| "unknown".to_string());

    let request = request.into_inner();

    // Validate password format
    if let Err(e) = valid_password(&request.password) {
        return Err(Status::invalid_argument(e.to_string()));
    }

    // Check if email is already in use
    match UserModel::check_email(&service.db, request.email.clone()).await {
        Ok(true) => {}
        Ok(false) => return Err(Status::invalid_argument("Email already in use".to_string())),
        Err(e) => return Err(Status::invalid_argument(e.to_string())),
    }

    let password_hash = SessionModel::create_password_hash(request.password.clone()).await?;

    // Create new user
    let user = service
        .db
        .create::<CreateUserModel, UserModel>(
            "users",
            CreateUserModel {
                email: request.email.clone(),
                password_hash,
            },
        )
        .await?
        .into_iter()
        .next()
        .ok_or_else(|| Status::internal("Failed to create user"))?;

    // Create session
    let session =
        SessionModel::create_session(&service.db, user.id.clone(), false, Some(ip_address))
            .await
            .map_err(|e| Status::internal(e.to_string()))?;

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
    use kiro_database::{db_bridge::MockDatabaseOperations, DatabaseError};
    use mockall::predicate::{always, eq};

    #[tokio::test]
    async fn test_register_success() {
        let mut mock_db = MockDatabaseOperations::new();
        let user = UserModel::default();

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

        let response = register(&service, request).await.unwrap().into_inner();

        assert!(!response.token.is_empty());
        assert!(response.expire_date.is_some());
    }

    #[tokio::test]
    async fn test_register_invalid_password() {
        let mock_db = MockDatabaseOperations::new();

        let service = AuthService {
            db: Database::Mock(mock_db),
        };

        let request = Request::new(AuthRequest {
            email: "test@example.com".to_string(),
            password: "invalid".to_string(),
        });

        let error = register(&service, request).await.unwrap_err();

        assert_eq!(error.code(), tonic::Code::InvalidArgument);
        assert_eq!(
            error.message(),
            "Password too short. Minimum size: 8 characters"
        );
    }

    #[tokio::test]
    async fn test_register_email_in_use() {
        let mut mock_db = MockDatabaseOperations::new();
        let user = UserModel::default();

        mock_db
            .expect_read_by_field::<UserModel>()
            .with(eq("users"), eq("email"), eq("test@example.com"), eq(None))
            .times(1)
            .returning(move |_, _, _, _| Ok(vec![user.clone()]));

        let service = AuthService {
            db: Database::Mock(mock_db),
        };

        let request = Request::new(AuthRequest {
            email: "test@example.com".to_string(),
            password: "Password123!".to_string(),
        });

        let error = register(&service, request).await.unwrap_err();
        assert_eq!(error.code(), tonic::Code::InvalidArgument);
        assert_eq!(error.message(), "Email already in use");
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

        let request = Request::new(AuthRequest {
            email: "test@example.com".to_string(),
            password: "Password123!".to_string(),
        });

        let error = register(&service, request).await.unwrap_err();
        assert_eq!(error.code(), tonic::Code::Internal);
        assert_eq!(error.message(), "Failed to create user");
    }

    #[tokio::test]
    async fn test_register_session_creation_failure() {
        let mut mock_db = MockDatabaseOperations::new();
        let user = UserModel::default();

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

        let request = Request::new(AuthRequest {
            email: "test@example.com".to_string(),
            password: "Password123!".to_string(),
        });

        let error = register(&service, request).await.unwrap_err();
        assert_eq!(error.code(), tonic::Code::Internal);
        assert_eq!(error.message(), "Internal error: Failed to create session");
    }
}
