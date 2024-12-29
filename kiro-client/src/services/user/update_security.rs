// services/user/update_security.rs
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

/// Updates a user's security settings
///
/// # Arguments
///
/// * `service` - The UserService instance
/// * `request` - The request containing the new security settings
///
/// # Returns
///
/// Returns an empty response on success
///
/// # Errors
///
/// Returns Status::unauthenticated if no valid session is found
/// Returns Status::invalid_argument if no security settings are provided
/// Returns Status::internal for database errors
///
/// # Example
///
/// ```rust,no_run
/// use tonic::{Request, Response, Status};
/// use kiro_api::client::v1::{client_service_server::ClientService, update_security_request::Value, UpdateSecurityRequest};
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
/// // Update user's security settings
/// let request = Request::new(UpdateSecurityRequest {
///    field: "two_factor".to_string(),
///    value: Some(Value::TwoFactor(true)),
/// });
///
///
/// // Async block to allow `await`
/// tokio::runtime::Runtime::new().unwrap().block_on(async {
///     ClientService::update_security(&service, request).await;
///
///     println!("Security settings updated");
/// });
/// ```
pub async fn update_security(
    service: &ClientService, request: Request<UpdateSecurityRequest>,
) -> Result<Response<Empty>, Status> {
    // Get authenticated session from request extensions
    let session = request
        .extensions()
        .get::<SessionModel>()
        .ok_or_else(|| Status::unauthenticated("No valid session found"))?;

    // Get the field and value from the request
    let field = request.get_ref().field.as_str();

    // QR code field is immutable
    if field == "qr_code" {
        return Err(Status::invalid_argument("QR code field is immutable"));
    }

    let value = match field {
        "two_factor" | "magic_link" => serde_json::Value::Bool(request.get_ref().value.is_some()),
        "qr_code" => serde_json::Value::String(request.get_ref().value.is_some().to_string()),
        _ => unreachable!(),
    };

    if !["two_factor", "qr_code", "magic_link"].contains(&field) {
        return Err(Status::invalid_argument("Invalid security field"));
    }

    // Update the security setting in the database
    service
        .db
        .update_field(
            session.user_id.clone(),
            &format!("settings/security/{}", field),
            value,
        )
        .await
        .map_err(|e| Status::internal(e.to_string()))?;

    Ok(Response::new(Empty {}))
}

#[cfg(test)]
mod tests {
    use super::*;

    use kiro_api::client::v1::update_security_request::Value as JsonValue;
    use kiro_database::{db_bridge::MockDatabaseOperations, DatabaseError};
    use mockall::predicate::eq;

    #[tokio::test]
    async fn test_update_security_two_factor_enable() {
        let mut mock_db = MockDatabaseOperations::new();
        let test_session = SessionModel::default();
        let user_id = test_session.user_id.clone();

        mock_db
            .expect_update_field()
            .with(
                eq(user_id),
                eq("settings/security/two_factor"),
                eq(serde_json::Value::Bool(true)),
            )
            .times(1)
            .returning(|_, _, _| Ok(()));

        let service = ClientService {
            db: Database::Mock(mock_db),
        };

        let mut request = Request::new(UpdateSecurityRequest {
            field: "two_factor".to_string(),
            value: Some(JsonValue::TwoFactor(true)),
        });
        request.extensions_mut().insert(test_session);

        let response = update_security(&service, request).await;
        assert!(response.is_ok());
    }

    #[tokio::test]
    async fn test_update_security_two_factor_disable() {
        let mut mock_db = MockDatabaseOperations::new();
        let test_session = SessionModel::default();
        let user_id = test_session.user_id.clone();

        mock_db
            .expect_update_field()
            .with(
                eq(user_id),
                eq("settings/security/two_factor"),
                eq(serde_json::Value::Bool(false)),
            )
            .times(1)
            .returning(|_, _, _| Ok(()));

        let service = ClientService {
            db: Database::Mock(mock_db),
        };

        let mut request = Request::new(UpdateSecurityRequest {
            field: "two_factor".to_string(),
            value: None,
        });
        request.extensions_mut().insert(test_session);

        let response = update_security(&service, request).await;
        assert!(response.is_ok());
    }

    #[tokio::test]
    async fn test_update_security_magic_link_enable() {
        let mut mock_db = MockDatabaseOperations::new();
        let test_session = SessionModel::default();
        let user_id = test_session.user_id.clone();

        mock_db
            .expect_update_field()
            .with(
                eq(user_id),
                eq("settings/security/magic_link"),
                eq(serde_json::Value::Bool(true)),
            )
            .times(1)
            .returning(|_, _, _| Ok(()));

        let service = ClientService {
            db: Database::Mock(mock_db),
        };

        let mut request = Request::new(UpdateSecurityRequest {
            field: "magic_link".to_string(),
            value: Some(JsonValue::MagicLink(true)),
        });
        request.extensions_mut().insert(test_session);

        let response = update_security(&service, request).await;
        assert!(response.is_ok());
    }

    #[tokio::test]
    async fn test_update_security_no_session() {
        let mock_db = MockDatabaseOperations::new();
        let service = ClientService {
            db: Database::Mock(mock_db),
        };

        let request = Request::new(UpdateSecurityRequest {
            field: "two_factor".to_string(),
            value: Some(JsonValue::TwoFactor(true)),
        });
        // Don't insert session into extensions

        let error = update_security(&service, request).await.unwrap_err();
        assert_eq!(error.code(), tonic::Code::Unauthenticated);
        assert_eq!(error.message(), "No valid session found");
    }

    #[tokio::test]
    async fn test_update_security_db_error() {
        let mut mock_db = MockDatabaseOperations::new();
        let test_session = SessionModel::default();
        let user_id = test_session.user_id.clone();

        mock_db
            .expect_update_field()
            .with(
                eq(user_id),
                eq("settings/security/two_factor"),
                eq(serde_json::Value::Bool(true)),
            )
            .times(1)
            .returning(|_, _, _| Err(DatabaseError::Internal("Database error".to_string())));

        let service = ClientService {
            db: Database::Mock(mock_db),
        };

        let mut request = Request::new(UpdateSecurityRequest {
            field: "two_factor".to_string(),
            value: Some(JsonValue::TwoFactor(true)),
        });
        request.extensions_mut().insert(test_session);

        let error = update_security(&service, request).await.unwrap_err();
        assert_eq!(error.code(), tonic::Code::Internal);
        assert!(error.message().contains("Database error"));
    }

    #[tokio::test]
    async fn test_update_security_admin_session() {
        let mut mock_db = MockDatabaseOperations::new();
        let mut admin_session = SessionModel::default();
        admin_session.is_admin = true;
        let user_id = admin_session.user_id.clone();

        mock_db
            .expect_update_field()
            .with(
                eq(user_id),
                eq("settings/security/two_factor"),
                eq(serde_json::Value::Bool(true)),
            )
            .times(1)
            .returning(|_, _, _| Ok(()));

        let service = ClientService {
            db: Database::Mock(mock_db),
        };

        let mut request = Request::new(UpdateSecurityRequest {
            field: "two_factor".to_string(),
            value: Some(JsonValue::TwoFactor(true)),
        });
        request.extensions_mut().insert(admin_session);

        let response = update_security(&service, request).await;
        assert!(response.is_ok());
    }

    #[tokio::test]
    async fn test_update_security_toggle_multiple_fields() {
        let mut mock_db = MockDatabaseOperations::new();
        let test_session = SessionModel::default();
        let user_id = test_session.user_id.clone();

        // First update: two_factor
        mock_db
            .expect_update_field()
            .with(
                eq(user_id.clone()),
                eq("settings/security/two_factor"),
                eq(serde_json::Value::Bool(true)),
            )
            .times(1)
            .returning(|_, _, _| Ok(()));

        // Second update: magic_link
        mock_db
            .expect_update_field()
            .with(
                eq(user_id.clone()),
                eq("settings/security/magic_link"),
                eq(serde_json::Value::Bool(false)),
            )
            .times(1)
            .returning(|_, _, _| Ok(()));

        let service = ClientService {
            db: Database::Mock(mock_db),
        };

        // Test two_factor update
        let mut request1 = Request::new(UpdateSecurityRequest {
            field: "two_factor".to_string(),
            value: Some(JsonValue::TwoFactor(true)),
        });
        request1.extensions_mut().insert(test_session.clone());
        let response1 = update_security(&service, request1).await;
        assert!(response1.is_ok());

        // Test magic_link update
        let mut request2 = Request::new(UpdateSecurityRequest {
            field: "magic_link".to_string(),
            value: None,
        });
        request2.extensions_mut().insert(test_session.clone());
        let response2 = update_security(&service, request2).await;
        assert!(response2.is_ok());
    }
}
