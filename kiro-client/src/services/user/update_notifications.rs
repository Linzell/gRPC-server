// services/user/update_notifications.rs
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

/// Updates a user's notification settings
///
/// # Arguments
///
/// * `service` - The UserService instance
/// * `request` - The request containing the new notification settings
///
/// # Returns
///
/// Returns an empty response on success
///
/// # Errors
///
/// Returns Status::unauthenticated if no valid session is found
/// Returns Status::invalid_argument if no notification settings are provided
/// Returns Status::internal for database errors
///
/// # Example
///
/// ```rust,no_run
/// use tonic::{Request, Response, Status};
/// use kiro_api::client::v1::{client_service_server::ClientService, UpdateNotificationsRequest};
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
/// // Update notifications request
/// let request = Request::new(UpdateNotificationsRequest {
///    field: "email".to_string(),
///    value: true,
/// });
///
/// // Async block to allow `await`
/// tokio::runtime::Runtime::new().unwrap().block_on(async {
///     ClientService::update_notifications(&service, request).await;
///
///     println!("Updated user's email notification settings");
/// });
/// ```
pub async fn update_notifications(
    service: &ClientService, request: Request<UpdateNotificationsRequest>,
) -> Result<Response<Empty>, Status> {
    // Get authenticated session from request extensions
    let session = request
        .extensions()
        .get::<SessionModel>()
        .ok_or_else(|| Status::unauthenticated("No valid session found"))?;

    // Get the field and value from the request
    let field = request.get_ref().field.as_str();
    let value = request.get_ref().value;

    // Validate the notification field name
    if !["email", "push", "sms"].contains(&field) {
        return Err(Status::invalid_argument(format!(
            "Invalid notification field: {}",
            field
        )));
    }

    // Update the user's notification settings in the database
    service
        .db
        .update_field(
            session.user_id.clone(),
            &format!("settings/notifications/{}", field),
            value,
        )
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
    async fn test_update_notifications_email_success() {
        let mut mock_db = MockDatabaseOperations::new();
        let test_session = SessionModel::default();
        let user_id = test_session.user_id.clone();

        mock_db
            .expect_update_field()
            .with(eq(user_id), eq("settings/notifications/email"), eq(true))
            .times(1)
            .returning(|_, _, _| Ok(()));

        let service = ClientService {
            db: Database::Mock(mock_db),
        };

        let mut request = Request::new(UpdateNotificationsRequest {
            field: "email".to_string(),
            value: true,
        });
        request.extensions_mut().insert(test_session);

        let response = update_notifications(&service, request).await;
        assert!(response.is_ok());
    }

    #[tokio::test]
    async fn test_update_notifications_push_success() {
        let mut mock_db = MockDatabaseOperations::new();
        let test_session = SessionModel::default();
        let user_id = test_session.user_id.clone();

        mock_db
            .expect_update_field()
            .with(eq(user_id), eq("settings/notifications/push"), eq(false))
            .times(1)
            .returning(|_, _, _| Ok(()));

        let service = ClientService {
            db: Database::Mock(mock_db),
        };

        let mut request = Request::new(UpdateNotificationsRequest {
            field: "push".to_string(),
            value: false,
        });
        request.extensions_mut().insert(test_session);

        let response = update_notifications(&service, request).await;
        assert!(response.is_ok());
    }

    #[tokio::test]
    async fn test_update_notifications_sms_success() {
        let mut mock_db = MockDatabaseOperations::new();
        let test_session = SessionModel::default();
        let user_id = test_session.user_id.clone();

        mock_db
            .expect_update_field()
            .with(eq(user_id), eq("settings/notifications/sms"), eq(true))
            .times(1)
            .returning(|_, _, _| Ok(()));

        let service = ClientService {
            db: Database::Mock(mock_db),
        };

        let mut request = Request::new(UpdateNotificationsRequest {
            field: "sms".to_string(),
            value: true,
        });
        request.extensions_mut().insert(test_session);

        let response = update_notifications(&service, request).await;
        assert!(response.is_ok());
    }

    #[tokio::test]
    async fn test_update_notifications_no_session() {
        let mock_db = MockDatabaseOperations::new();
        let service = ClientService {
            db: Database::Mock(mock_db),
        };

        let request = Request::new(UpdateNotificationsRequest {
            field: "email".to_string(),
            value: true,
        });
        // Don't insert session into extensions

        let error = update_notifications(&service, request).await.unwrap_err();
        assert_eq!(error.code(), tonic::Code::Unauthenticated);
        assert_eq!(error.message(), "No valid session found");
    }

    #[tokio::test]
    async fn test_update_notifications_invalid_field() {
        let mock_db = MockDatabaseOperations::new();
        let test_session = SessionModel::default();

        let service = ClientService {
            db: Database::Mock(mock_db),
        };

        let mut request = Request::new(UpdateNotificationsRequest {
            field: "invalid_field".to_string(),
            value: true,
        });
        request.extensions_mut().insert(test_session);

        let error = update_notifications(&service, request).await.unwrap_err();
        assert_eq!(error.code(), tonic::Code::InvalidArgument);
        assert!(error.message().contains("Invalid notification field"));
    }

    #[tokio::test]
    async fn test_update_notifications_db_error() {
        let mut mock_db = MockDatabaseOperations::new();
        let test_session = SessionModel::default();
        let user_id = test_session.user_id.clone();

        mock_db
            .expect_update_field()
            .with(eq(user_id), eq("settings/notifications/email"), eq(true))
            .times(1)
            .returning(|_, _, _| Err(DatabaseError::Internal("Database error".to_string())));

        let service = ClientService {
            db: Database::Mock(mock_db),
        };

        let mut request = Request::new(UpdateNotificationsRequest {
            field: "email".to_string(),
            value: true,
        });
        request.extensions_mut().insert(test_session);

        let error = update_notifications(&service, request).await.unwrap_err();
        assert_eq!(error.code(), tonic::Code::Internal);
        assert!(error.message().contains("Database error"));
    }

    #[tokio::test]
    async fn test_update_notifications_admin_session() {
        let mut mock_db = MockDatabaseOperations::new();
        let mut admin_session = SessionModel::default();
        admin_session.is_admin = true;
        let user_id = admin_session.user_id.clone();

        mock_db
            .expect_update_field()
            .with(eq(user_id), eq("settings/notifications/email"), eq(true))
            .times(1)
            .returning(|_, _, _| Ok(()));

        let service = ClientService {
            db: Database::Mock(mock_db),
        };

        let mut request = Request::new(UpdateNotificationsRequest {
            field: "email".to_string(),
            value: true,
        });
        request.extensions_mut().insert(admin_session);

        let response = update_notifications(&service, request).await;
        assert!(response.is_ok());
    }

    #[tokio::test]
    async fn test_update_notifications_empty_field() {
        let mock_db = MockDatabaseOperations::new();
        let test_session = SessionModel::default();

        let service = ClientService {
            db: Database::Mock(mock_db),
        };

        let mut request = Request::new(UpdateNotificationsRequest {
            field: "".to_string(),
            value: true,
        });
        request.extensions_mut().insert(test_session);

        let error = update_notifications(&service, request).await.unwrap_err();
        assert_eq!(error.code(), tonic::Code::InvalidArgument);
        assert!(error.message().contains("Invalid notification field"));
    }
}
