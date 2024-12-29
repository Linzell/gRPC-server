// services/user/update_theme.rs
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

/// Updates a user's theme preference
///
/// # Arguments
///
/// * `service` - The UserService instance
/// * `request` - The request containing the new theme
///
/// # Returns
///
/// Returns an empty response on success
///
/// # Errors
///
/// Returns Status::unauthenticated if no valid session is found
/// Returns Status::invalid_argument if no theme is provided
/// Returns Status::internal for database errors
///
/// # Example
///
/// ```rust,no_run
/// use tonic::{Request, Response, Status};
/// use kiro_api::client::v1::{client_service_server::ClientService, UpdateThemeRequest};
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
/// // Update user's theme request
/// let request = Request::new(UpdateThemeRequest {
///    theme: 0,    // 0 for light theme, 1 for dark theme, 2 for system theme
/// });
///
/// // Async block to allow `await`
/// tokio::runtime::Runtime::new().unwrap().block_on(async {
///     ClientService::update_theme(&service, request).await;
///
///     println!("User theme updated successfully");
/// });
/// ```
pub async fn update_theme(
    service: &ClientService, request: Request<UpdateThemeRequest>,
) -> Result<Response<Empty>, Status> {
    // Get authenticated session from request extensions
    let session = request
        .extensions()
        .get::<SessionModel>()
        .ok_or_else(|| Status::unauthenticated("No valid session found"))?;

    // Get the new theme from the request
    let theme = request.get_ref().theme;

    // Update the user's theme in the database
    service
        .db
        .update_field(session.user_id.clone(), "theme", theme)
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
    async fn test_update_theme_light_success() {
        let mut mock_db = MockDatabaseOperations::new();
        let test_session = SessionModel::default();
        let user_id = test_session.user_id.clone();

        mock_db
            .expect_update_field()
            .with(eq(user_id), eq("theme"), eq(0)) // Assuming 0 represents light theme
            .times(1)
            .returning(|_, _, _| Ok(()));

        let service = ClientService {
            db: Database::Mock(mock_db),
        };

        let mut request = Request::new(UpdateThemeRequest { theme: 0 });
        request.extensions_mut().insert(test_session);

        let response = update_theme(&service, request).await;
        assert!(response.is_ok());
    }

    #[tokio::test]
    async fn test_update_theme_dark_success() {
        let mut mock_db = MockDatabaseOperations::new();
        let test_session = SessionModel::default();
        let user_id = test_session.user_id.clone();

        mock_db
            .expect_update_field()
            .with(eq(user_id), eq("theme"), eq(1)) // Assuming 1 represents dark theme
            .times(1)
            .returning(|_, _, _| Ok(()));

        let service = ClientService {
            db: Database::Mock(mock_db),
        };

        let mut request = Request::new(UpdateThemeRequest { theme: 1 });
        request.extensions_mut().insert(test_session);

        let response = update_theme(&service, request).await;
        assert!(response.is_ok());
    }

    #[tokio::test]
    async fn test_update_theme_no_session() {
        let mock_db = MockDatabaseOperations::new();
        let service = ClientService {
            db: Database::Mock(mock_db),
        };

        let request = Request::new(UpdateThemeRequest { theme: 0 });
        // Don't insert session into extensions

        let error = update_theme(&service, request).await.unwrap_err();
        assert_eq!(error.code(), tonic::Code::Unauthenticated);
        assert_eq!(error.message(), "No valid session found");
    }

    #[tokio::test]
    async fn test_update_theme_db_error() {
        let mut mock_db = MockDatabaseOperations::new();
        let test_session = SessionModel::default();
        let user_id = test_session.user_id.clone();

        mock_db
            .expect_update_field()
            .with(eq(user_id), eq("theme"), eq(0))
            .times(1)
            .returning(|_, _, _| Err(DatabaseError::Internal("Database error".to_string())));

        let service = ClientService {
            db: Database::Mock(mock_db),
        };

        let mut request = Request::new(UpdateThemeRequest { theme: 0 });
        request.extensions_mut().insert(test_session);

        let error = update_theme(&service, request).await.unwrap_err();
        assert_eq!(error.code(), tonic::Code::Internal);
        assert!(error.message().contains("Database error"));
    }

    #[tokio::test]
    async fn test_update_theme_admin_session() {
        let mut mock_db = MockDatabaseOperations::new();
        let mut admin_session = SessionModel::default();
        admin_session.is_admin = true;
        let user_id = admin_session.user_id.clone();

        mock_db
            .expect_update_field()
            .with(eq(user_id), eq("theme"), eq(0))
            .times(1)
            .returning(|_, _, _| Ok(()));

        let service = ClientService {
            db: Database::Mock(mock_db),
        };

        let mut request = Request::new(UpdateThemeRequest { theme: 0 });
        request.extensions_mut().insert(admin_session);

        let response = update_theme(&service, request).await;
        assert!(response.is_ok());
    }

    #[tokio::test]
    async fn test_update_theme_toggle() {
        let mut mock_db = MockDatabaseOperations::new();
        let test_session = SessionModel::default();
        let user_id = test_session.user_id.clone();

        // First update: light theme
        mock_db
            .expect_update_field()
            .with(eq(user_id.clone()), eq("theme"), eq(0))
            .times(1)
            .returning(|_, _, _| Ok(()));

        // Second update: dark theme
        mock_db
            .expect_update_field()
            .with(eq(user_id), eq("theme"), eq(1))
            .times(1)
            .returning(|_, _, _| Ok(()));

        let service = ClientService {
            db: Database::Mock(mock_db),
        };

        // Test light theme update
        let mut request1 = Request::new(UpdateThemeRequest { theme: 0 });
        request1.extensions_mut().insert(test_session.clone());
        let response1 = update_theme(&service, request1).await;
        assert!(response1.is_ok());

        // Test dark theme update
        let mut request2 = Request::new(UpdateThemeRequest { theme: 1 });
        request2.extensions_mut().insert(test_session);
        let response2 = update_theme(&service, request2).await;
        assert!(response2.is_ok());
    }

    #[tokio::test]
    async fn test_update_theme_system_success() {
        let mut mock_db = MockDatabaseOperations::new();
        let test_session = SessionModel::default();
        let user_id = test_session.user_id.clone();

        mock_db
            .expect_update_field()
            .with(eq(user_id), eq("theme"), eq(2)) // Assuming 2 represents system theme
            .times(1)
            .returning(|_, _, _| Ok(()));

        let service = ClientService {
            db: Database::Mock(mock_db),
        };

        let mut request = Request::new(UpdateThemeRequest { theme: 2 });
        request.extensions_mut().insert(test_session);

        let response = update_theme(&service, request).await;
        assert!(response.is_ok());
    }

    #[tokio::test]
    async fn test_update_theme_multiple_changes() {
        let mut mock_db = MockDatabaseOperations::new();
        let test_session = SessionModel::default();
        let user_id = test_session.user_id.clone();

        // Multiple theme changes in sequence
        for theme in [0, 1, 2, 0] {
            mock_db
                .expect_update_field()
                .with(eq(user_id.clone()), eq("theme"), eq(theme))
                .times(1)
                .returning(|_, _, _| Ok(()));
        }

        let service = ClientService {
            db: Database::Mock(mock_db),
        };

        for theme in [0, 1, 2, 0] {
            let mut request = Request::new(UpdateThemeRequest { theme });
            request.extensions_mut().insert(test_session.clone());
            let response = update_theme(&service, request).await;
            assert!(response.is_ok());
        }
    }
}
