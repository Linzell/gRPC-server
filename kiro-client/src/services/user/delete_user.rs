// services/user/delete_user.rs
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

use kiro_database::db_bridge::DatabaseOperations;
use tonic::{Request, Response, Status};

use crate::SessionModel;

/// Deletes a user account
///
/// # Arguments
///
/// * `service` - The UserService instance
/// * `request` - The request containing the user's password
///
/// # Returns
///
/// Returns an empty response on success
///
/// # Errors
///
/// Returns Status::unauthenticated if no valid session is found
/// Returns Status::invalid_argument if no password is provided
/// Returns Status::internal for database errors
///
/// # Example
///
/// ```rust,no_run
/// use tonic::{Request, Response, Status};
/// use kiro_api::{client::v1::client_service_server::ClientService, google::protobuf::Empty};
/// use kiro_database::db_bridge::{Database, MockDatabaseOperations};
///
/// // Mock database
/// let mock_db = MockDatabaseOperations::new();
///
/// // Mock service
/// let service = kiro_client::ClientService {
///     db: Database::Mock(mock_db),
/// };
///
/// // Delete user request
/// let request = Request::new(Empty {});
///
/// // Async block to allow `await`
/// tokio::runtime::Runtime::new().unwrap().block_on(async {
///     ClientService::delete_user(&service, request).await;
///
///     println!("User account deleted");
/// });
/// ```
pub async fn delete_user(
    service: &ClientService, request: Request<Empty>,
) -> Result<Response<Empty>, Status> {
    // Get authenticated session from request extensions
    let session = request
        .extensions()
        .get::<SessionModel>()
        .ok_or_else(|| Status::unauthenticated("No valid session found"))?;

    // Delete the user account
    service
        .db
        .delete(session.user_id.clone())
        .await
        .map_err(|_| Status::internal("Failed to delete user account"))?;

    Ok(Response::new(Empty {}))
}

#[cfg(test)]
mod tests {
    use super::*;

    use kiro_database::{db_bridge::MockDatabaseOperations, DatabaseError};
    use mockall::predicate::eq;

    #[tokio::test]
    async fn test_delete_user_success() {
        let mut mock_db = MockDatabaseOperations::new();
        let test_session = SessionModel::default();
        let user_id = test_session.user_id.clone();

        // Expect delete operation to be called with the user's ID
        mock_db
            .expect_delete()
            .with(eq(user_id))
            .times(1)
            .returning(|_| Ok(Some(())));

        let service = ClientService {
            db: Database::Mock(mock_db),
        };

        let mut request = Request::new(Empty {});
        request.extensions_mut().insert(test_session);

        let response = delete_user(&service, request).await;
        assert!(response.is_ok());
    }

    #[tokio::test]
    async fn test_delete_user_no_session() {
        let mock_db = MockDatabaseOperations::new();
        let service = ClientService {
            db: Database::Mock(mock_db),
        };

        let request = Request::new(Empty {});
        // Don't insert session into extensions

        let error = delete_user(&service, request).await.unwrap_err();
        assert_eq!(error.code(), tonic::Code::Unauthenticated);
        assert_eq!(error.message(), "No valid session found");
    }

    #[tokio::test]
    async fn test_delete_user_db_error() {
        let mut mock_db = MockDatabaseOperations::new();
        let test_session = SessionModel::default();
        let user_id = test_session.user_id.clone();

        // Simulate database error
        mock_db
            .expect_delete()
            .with(eq(user_id))
            .times(1)
            .returning(|_| Err(DatabaseError::Internal("Database error".to_string())));

        let service = ClientService {
            db: Database::Mock(mock_db),
        };

        let mut request = Request::new(Empty {});
        request.extensions_mut().insert(test_session);

        let error = delete_user(&service, request).await.unwrap_err();
        assert_eq!(error.code(), tonic::Code::Internal);
        assert_eq!(error.message(), "Failed to delete user account");
    }

    #[tokio::test]
    async fn test_delete_user_admin_session() {
        let mut mock_db = MockDatabaseOperations::new();
        let mut admin_session = SessionModel::default();
        admin_session.is_admin = true;
        let user_id = admin_session.user_id.clone();

        mock_db
            .expect_delete()
            .with(eq(user_id))
            .times(1)
            .returning(|_| Ok(Some(())));

        let service = ClientService {
            db: Database::Mock(mock_db),
        };

        let mut request = Request::new(Empty {});
        request.extensions_mut().insert(admin_session);

        let response = delete_user(&service, request).await;
        assert!(response.is_ok());
    }
}
