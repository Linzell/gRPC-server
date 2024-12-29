// services/auth/logout.rs
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

use tonic::{Request, Response, Status};

use crate::SessionModel;

/// Logout service implementation
///
/// # Description
/// Logs out the current user by deleting the session from the database
///
/// # Arguments
/// * `service` - Reference to the authentication service
/// * `request` - The request containing the session to delete
///
/// # Returns
/// * `Ok(Empty)` - Empty response on success
/// * `Err(Status)` - Appropriate error status on failure
///
/// # Errors
/// * `INTERNAL_SERVER_ERROR` - Database error
/// * `NOT_FOUND` - Session not found in request
///
/// # Example
/// ```rust,no_run
/// use tonic::{Request, Response, Status};
/// use kiro_api::{auth::v1::auth_service_server::AuthService, google::protobuf::Empty};
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
/// // Logout request
/// let request = Request::new(Empty {});
///
/// // Async block to allow `await`
/// tokio::runtime::Runtime::new().unwrap().block_on(async {
///     AuthService::logout(&service, request).await;
///
///     println!("Login successful");
/// });
/// ```
pub async fn logout(
    service: &AuthService, request: Request<Empty>,
) -> Result<Response<Empty>, Status> {
    let session = request
        .extensions()
        .get::<SessionModel>()
        .ok_or_else(|| Status::not_found("Session not found in request"))?;

    match SessionModel::delete_session(&service.db, session.id.clone()).await {
        Ok(_) => Ok(Response::new(Empty {})),
        Err(e) => Err(Status::internal(e.to_string())),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use kiro_database::db_bridge::MockDatabaseOperations;
    use mockall::predicate::eq;

    #[tokio::test]
    async fn test_logout_success() {
        let mut mock_db = MockDatabaseOperations::new();
        let session = SessionModel::default();

        mock_db
            .expect_delete()
            .with(eq(session.id.clone()))
            .times(1)
            .returning(|_| Ok(Some(())));

        let service = AuthService {
            db: Database::Mock(mock_db),
        };

        let mut request = Request::new(Empty {});
        request.extensions_mut().insert(session);

        let response = logout(&service, request).await;
        assert!(response.is_ok());
    }

    #[tokio::test]
    async fn test_logout_no_session() {
        let mock_db = MockDatabaseOperations::new();
        let service = AuthService {
            db: Database::Mock(mock_db),
        };

        let request = Request::new(Empty {});

        let response = logout(&service, request).await;
        assert!(response.is_err());
    }
}
