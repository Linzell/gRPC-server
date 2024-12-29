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
/// // Read user request
/// let request = Request::new(Empty {});
///
/// // Async block to allow `await`
/// tokio::runtime::Runtime::new().unwrap().block_on(async {
///     ClientService::read_user(&service, request).await;
///
///     println!("User data streamed");
/// });
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

    use futures::StreamExt;
    use kiro_database::{db_bridge::MockDatabaseOperations, DatabaseError, DbId};
    use mockall::predicate::eq;
    use std::pin::Pin;
    use tokio_stream::wrappers::ReceiverStream;

    #[tokio::test]
    #[ignore]
    async fn test_read_user_initial_success() {
        let mut mock_db = MockDatabaseOperations::new();
        let test_session = SessionModel::default();
        let test_user = UserModel::default();

        // Mock initial select with correct table name
        mock_db
            .expect_select()
            .with(eq(DbId::default()))
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
        let test_session = SessionModel::default();

        // Mock select with correct table name
        mock_db
            .expect_select::<UserModel>()
            .with(eq(DbId::default()))
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
        let test_session = SessionModel::default();

        // Mock select with correct table name
        mock_db
            .expect_select::<UserModel>()
            .with(eq(DbId::default()))
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
