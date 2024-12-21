// services/logout.rs
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

/// Handles user logout by deleting the current session
///
/// # Arguments
///
/// * `service` - Reference to the authentication service
/// * `request` - The incoming request containing session information
///
/// # Returns
///
/// Returns empty response on success, or appropriate error status
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

    use chrono::Utc;
    use kiro_database::{db_bridge::MockDatabaseOperations, DbDateTime, DbId};
    use mockall::predicate::eq;

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
    async fn test_logout_success() {
        let mut mock_db = MockDatabaseOperations::new();
        let session = create_test_session();

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
