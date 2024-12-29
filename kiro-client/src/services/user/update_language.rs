// services/user/update_language.rs
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

/// Updates a user's language preference
///
/// # Arguments
///
/// * `service` - The UserService instance
/// * `request` - The request containing the new language
///
/// # Returns
///
/// Returns an empty response on success
///
/// # Errors
///
/// Returns Status::unauthenticated if no valid session is found
/// Returns Status::invalid_argument if no language is provided
/// Returns Status::internal for database errors
///
/// # Example
///
/// ```rust,no_run
/// use tonic::{Request, Response, Status};
/// use kiro_api::client::v1::{client_service_server::ClientService, UpdateLanguageRequest};
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
/// // Update email request
/// let request = Request::new(UpdateLanguageRequest {
///     language: 0, // English
/// });
///
///
/// // Async block to allow `await`
/// tokio::runtime::Runtime::new().unwrap().block_on(async {
///     ClientService::update_language(&service, request).await;
///
///     println!("Language updated successfully");
/// });
/// ```
pub async fn update_language(
    service: &ClientService, request: Request<UpdateLanguageRequest>,
) -> Result<Response<Empty>, Status> {
    // Get authenticated session from request extensions
    let session = request
        .extensions()
        .get::<SessionModel>()
        .ok_or_else(|| Status::unauthenticated("No valid session found"))?;

    // Get the new language from the request
    let language = request.get_ref().language;

    // Check if the language is a valid Language enum value
    if !matches!(language, 0..=9) {
        return Err(Status::invalid_argument("Invalid language value"));
    }

    // Update the user's language in the database
    service
        .db
        .update_field(session.user_id.clone(), "language", language)
        .await
        .map_err(|e| Status::internal(e.to_string()))?;

    Ok(Response::new(Empty {}))
}

#[cfg(test)]
mod tests {
    use super::*;

    use kiro_database::{db_bridge::MockDatabaseOperations, DatabaseError};
    use mockall::predicate::eq;

    #[tokio::test]
    async fn test_update_language_success() {
        let mut mock_db = MockDatabaseOperations::new();
        let test_session = SessionModel::default();
        let user_id = test_session.user_id.clone();

        mock_db
            .expect_update_field()
            .with(eq(user_id), eq("language"), eq(0))
            .times(1)
            .returning(|_, _, _| Ok(()));

        let service = ClientService {
            db: Database::Mock(mock_db),
        };

        let mut request = Request::new(UpdateLanguageRequest { language: 0 });
        request.extensions_mut().insert(test_session);

        let response = update_language(&service, request).await;
        assert!(response.is_ok());
    }

    #[tokio::test]
    async fn test_update_language_no_session() {
        let mock_db = MockDatabaseOperations::new();
        let service = ClientService {
            db: Database::Mock(mock_db),
        };

        let request = Request::new(UpdateLanguageRequest { language: 0 });
        // Don't insert session into extensions

        let error = update_language(&service, request).await.unwrap_err();
        assert_eq!(error.code(), tonic::Code::Unauthenticated);
        assert_eq!(error.message(), "No valid session found");
    }

    #[tokio::test]
    async fn test_update_language_db_error() {
        let mut mock_db = MockDatabaseOperations::new();
        let test_session = SessionModel::default();
        let user_id = test_session.user_id.clone();

        // Simulate database error
        mock_db
            .expect_update_field()
            .with(eq(user_id), eq("language"), eq(0))
            .times(1)
            .returning(|_, _, _| Err(DatabaseError::Internal("Database error".to_string())));

        let service = ClientService {
            db: Database::Mock(mock_db),
        };

        let mut request = Request::new(UpdateLanguageRequest { language: 0 });
        request.extensions_mut().insert(test_session);

        let error = update_language(&service, request).await.unwrap_err();
        assert_eq!(error.code(), tonic::Code::Internal);
        assert!(error.message().contains("Database error"));
    }

    #[tokio::test]
    async fn test_update_language_admin_session() {
        let mut mock_db = MockDatabaseOperations::new();
        let mut admin_session = SessionModel::default();
        admin_session.is_admin = true;
        let user_id = admin_session.user_id.clone();

        mock_db
            .expect_update_field()
            .with(eq(user_id), eq("language"), eq(0))
            .times(1)
            .returning(|_, _, _| Ok(()));

        let service = ClientService {
            db: Database::Mock(mock_db),
        };

        let mut request = Request::new(UpdateLanguageRequest { language: 0 });
        request.extensions_mut().insert(admin_session);

        let response = update_language(&service, request).await;
        assert!(response.is_ok());
    }

    #[tokio::test]
    async fn test_update_language_invalid_language() {
        let mock_db = MockDatabaseOperations::new();
        let test_session = SessionModel::default();

        let service = ClientService {
            db: Database::Mock(mock_db),
        };

        let mut request = Request::new(UpdateLanguageRequest { language: 99 });
        request.extensions_mut().insert(test_session);

        let error = update_language(&service, request).await.unwrap_err();
        assert_eq!(error.code(), tonic::Code::InvalidArgument);
        assert_eq!(error.message(), "Invalid language value");
    }
}
