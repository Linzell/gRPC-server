// services/user/read_user.rs
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

use std::pin::Pin;

use futures::{future, Stream, StreamExt};
use kiro_api::client::v1::{Settings, User};
use kiro_database::{db_bridge::DatabaseOperations, DbId};
use tokio::sync::mpsc;
use tokio_stream::wrappers::ReceiverStream;
use tonic::{Request, Response, Status};

use crate::models::{SessionModel, UserModel};

pub type ReadUserStream = Pin<Box<dyn Stream<Item = Result<User, Status>> + Send + Sync>>;

/// # Read user
///
/// Streams user data with live updates.
/// Returns the authenticated user's data and streams updates when changes occur.
///
/// ## Arguments
///
/// * `service` - The user service instance containing the database connection
/// * `request` - The incoming request containing the user's session
///
/// ## Returns
///
/// Returns a streaming response of `User` containing user information
///
/// ## Examples
///
/// ```rust, ignore
/// let request = Request::new(ReadUserRequest {});
/// let response = read_user(&service, request).await?;
/// let mut stream = response.into_inner();
/// while let Some(user_data) = stream.next().await {
///     println!("Received user update: {:?}", user_data);
/// }
/// ```
pub async fn read_user(
    service: &ClientService, request: Request<Empty>,
) -> Result<Response<ReadUserStream>, Status> {
    let (tx, rx) = mpsc::channel(32);

    let session = request
        .extensions()
        .get::<SessionModel>()
        .ok_or_else(|| Status::unauthenticated("Missing session information"))?;

    let db = service.db.clone();
    let user_id = session.user_id.clone();

    tokio::spawn(async move {
        let _ = stream_user(&db, &tx, user_id).await;
    });

    let stream = ReceiverStream::new(rx)
        .map(|result| result)
        .take_while(|result| future::ready(result.is_ok()));

    Ok(Response::new(Box::pin(stream)))
}

/// Handles the streaming of user data from the database
///
/// ## Arguments
///
/// * `db` - Database connection for querying user data
/// * `tx` - Channel sender for streaming updates
/// * `user_id` - ID of the user to stream
///
/// ## Returns
///
/// Returns Ok(()) on successful completion or an error if streaming fails
async fn stream_user(
    db: &Database, tx: &mpsc::Sender<Result<User, Status>>, user_id: DbId,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    // Initial read
    match db.select::<UserModel>(user_id.clone()).await {
        Ok(Some(user)) => send_user_update(tx, user).await?,
        Ok(None) => {
            tx.send(Err(Status::not_found("User not found"))).await?;
            return Ok(());
        }
        Err(e) => {
            tx.send(Err(Status::internal(e.to_string()))).await?;
            return Ok(());
        }
    }

    // Live updates
    let mut live_stream = db.live::<UserModel>("users").await?;

    while let Some(result) = live_stream.next().await {
        match result {
            Ok(users) => {
                for user in users {
                    if user.id == user_id {
                        send_user_update(tx, user).await?;
                    }
                }
            }
            Err(e) => {
                tx.send(Err(Status::internal(e.to_string()))).await?;
                break;
            }
        }
    }

    Ok(())
}

/// Sends a user update through the channel
///
/// ## Arguments
///
/// * `db` - Database connection for querying related data
/// * `tx` - Channel sender for the update
/// * `user` - User model to send
///
/// ## Returns
///
/// Returns Ok(()) on successful send or an error if sending fails
async fn send_user_update(
    tx: &mpsc::Sender<Result<User, Status>>, user: UserModel,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let user = User {
        email: user.email,
        avatar: user.avatar,
        settings: Some(Settings::from(&user.settings)),
        is_admin: user.is_admin,
    };

    tx.send(Ok(user)).await?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        Language, NotificationSettings, PrivacySettings, SecuritySettings, Theme, UserSettings,
    };
    use chrono::Utc;
    use futures::StreamExt;
    use kiro_database::{db_bridge::MockDatabaseOperations, DatabaseError, DbDateTime, DbId};
    use mockall::predicate::eq;
    use std::pin::Pin;
    use tokio_stream::wrappers::ReceiverStream;

    fn create_test_session() -> SessionModel {
        SessionModel {
            id: DbId::from(("sessions", "1")),
            session_key: "session_token".to_string(),
            expires_at: DbDateTime::from(Utc::now() + chrono::Duration::days(2)),
            user_id: DbId::from(("users", "123")),
            ip_address: Some("127.0.0.1".to_string()),
            is_admin: false,
        }
    }

    fn create_test_user() -> UserModel {
        UserModel {
            id: DbId::from(("users", "123")),
            customer_id: None,
            email: "test@example.com".to_string(),
            password_hash: "hash".to_string(),
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
            groups: vec![DbId::from(("groups", "1"))],
            created_at: DbDateTime::from(Utc::now()),
            updated_at: DbDateTime::from(Utc::now()),
            activated: true,
            is_admin: false,
        }
    }

    #[tokio::test]
    #[ignore]
    async fn test_read_user_initial_success() {
        let mut mock_db = MockDatabaseOperations::new();
        let test_session = create_test_session();
        let test_user = create_test_user();

        // Mock initial select with correct table name
        mock_db
            .expect_select()
            .with(eq(DbId::from(("users", "123"))))
            .times(1)
            .returning(move |_| Ok(Some(test_user.clone())));

        // Mock live stream
        mock_db
            .expect_live()
            .with(eq("users"))
            .times(1)
            .returning(move |_| {
                let (_tx, rx) = mpsc::channel(32);
                let rx = ReceiverStream::new(rx);
                Ok(Box::pin(rx)
                    as Pin<
                        Box<dyn Stream<Item = Result<Vec<UserModel>, DatabaseError>> + Send + Sync>,
                    >)
            });

        let service = ClientService {
            db: Database::Mock(mock_db),
        };

        let mut request = Request::new(Empty {});
        request.extensions_mut().insert(test_session);

        let response = read_user(&service, request).await.unwrap();
        let mut stream = response.into_inner();

        // Test initial update
        if let Some(result) = stream.next().await {
            let user = result.unwrap();
            assert_eq!(user.email, "test@example.com");
            assert_eq!(user.avatar.unwrap(), "avatar.jpg");
            assert!(!user.is_admin);
        } else {
            panic!("No initial user data received");
        }
    }

    #[tokio::test]
    async fn test_read_user_no_session() {
        let mock_db = MockDatabaseOperations::new();
        let service = ClientService {
            db: Database::Mock(mock_db),
        };

        let request = Request::new(Empty {});
        // Don't insert session into extensions

        let status = read_user(&service, request).await.err().unwrap();
        assert_eq!(status.code(), tonic::Code::Unauthenticated);
        assert_eq!(status.message(), "Missing session information");
    }

    #[tokio::test]
    #[ignore]
    async fn test_read_user_not_found() {
        let mut mock_db = MockDatabaseOperations::new();
        let test_session = create_test_session();

        // Mock select with correct table name
        mock_db
            .expect_select::<UserModel>()
            .with(eq(DbId::from(("users", "123"))))
            .times(1)
            .returning(|_| Ok(None));

        // Mock live stream
        mock_db
            .expect_live()
            .with(eq("users"))
            .times(1)
            .returning(|_| {
                let (_tx, rx) = mpsc::channel(32);
                let rx = ReceiverStream::new(rx);
                Ok(Box::pin(rx)
                    as Pin<
                        Box<dyn Stream<Item = Result<Vec<UserModel>, DatabaseError>> + Send + Sync>,
                    >)
            });

        let service = ClientService {
            db: Database::Mock(mock_db),
        };

        let mut request = Request::new(Empty {});
        request.extensions_mut().insert(test_session);

        let response = read_user(&service, request).await.unwrap();
        let mut stream = response.into_inner();

        let error = stream.next().await.unwrap().unwrap_err();
        assert_eq!(error.code(), tonic::Code::NotFound);
        assert_eq!(error.message(), "User not found");
    }

    #[tokio::test]
    #[ignore]
    async fn test_read_user_db_error() {
        let mut mock_db = MockDatabaseOperations::new();
        let test_session = create_test_session();

        // Mock select with correct table name
        mock_db
            .expect_select::<UserModel>()
            .with(eq(DbId::from(("users", "123"))))
            .times(1)
            .returning(|_| Err(DatabaseError::Internal("Database error".to_string())));

        // Mock live stream
        mock_db
            .expect_live()
            .with(eq("users"))
            .times(1)
            .returning(|_| {
                let (_tx, rx) = mpsc::channel(32);
                let rx = ReceiverStream::new(rx);
                Ok(Box::pin(rx)
                    as Pin<
                        Box<dyn Stream<Item = Result<Vec<UserModel>, DatabaseError>> + Send + Sync>,
                    >)
            });

        let service = ClientService {
            db: Database::Mock(mock_db),
        };

        let mut request = Request::new(Empty {});
        request.extensions_mut().insert(test_session);

        let response = read_user(&service, request).await.unwrap();
        let mut stream = response.into_inner();

        let error = stream.next().await.unwrap().unwrap_err();
        assert_eq!(error.code(), tonic::Code::Internal);
        assert!(error.message().contains("Database error"));
    }
}
