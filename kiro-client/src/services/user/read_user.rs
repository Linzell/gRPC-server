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
        .ok_or_else(|| Status::internal("Missing session information"))?;

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
