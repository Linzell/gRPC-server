// http/user/read_user.rs
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

use axum::{
    extract::State,
    http::StatusCode,
    response::{sse::Event, IntoResponse, Sse},
    Extension, Json,
};
use futures::StreamExt;
use kiro_api::client::v1::{Settings, User};
use kiro_database::{db_bridge::DatabaseOperations, DbId};
use tokio::sync::mpsc;
use tokio_stream::wrappers::ReceiverStream;

use crate::models::{SessionModel, UserModel};

/// User read route webhook
///
/// # Description
/// Streams the current user's data
///
/// # Arguments
/// * `service` - The client service instance
/// * `session` - The current session model
///
/// # Returns
/// * HTTP response with either:
///   * `200 OK` with the current user's data stream
///   * Error status code with message
///
/// # Errors
/// * `401 UNAUTHORIZED` - No session or invalid session
/// * `404 NOT FOUND` - User not found
/// * `500 INTERNAL SERVER ERROR` - Database or server error
///
/// # Example
/// ```rust,ignore
/// use axum::{Extension};
/// use kiro_api::session::SessionModel;
///
/// let session = SessionModel::default();
/// let response = read_user(&service, &session).await?;
/// ```
pub async fn read_user(
    State(service): State<ClientService>, Extension(session): Extension<SessionModel>,
) -> impl IntoResponse {
    let (tx, rx) = mpsc::channel(32);

    tokio::spawn(async move {
        let _ = stream_user(&service.db, &tx, session.user_id).await;
    });

    let stream = ReceiverStream::new(rx);

    Sse::new(stream.map(|result| match result {
        Ok(user) => Ok::<_, Box<dyn std::error::Error + Send + Sync>>(
            Event::default().data(serde_json::to_string(&user).unwrap()),
        ),
        Err((_status, json)) => {
            let error_msg = json
                .0
                .get("error")
                .and_then(|v| v.as_str())
                .unwrap_or("Unknown error");
            Ok::<_, Box<dyn std::error::Error + Send + Sync>>(
                Event::default().data(format!("error: {}", error_msg)),
            )
        }
    }))
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
    db: &Database, tx: &mpsc::Sender<Result<User, (StatusCode, Json<serde_json::Value>)>>,
    user_id: DbId,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    // Initial read
    match db.select::<UserModel>(user_id.clone()).await {
        Ok(Some(user)) => send_user_update(tx, user).await?,
        Ok(None) => {
            tx.send(Err((
                StatusCode::NOT_FOUND,
                Json(serde_json::json!({ "error": "User not found" })),
            )))
            .await?;
            return Ok(());
        }
        Err(e) => {
            tx.send(Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({ "error": e.to_string() })),
            )))
            .await?;
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
                tx.send(Err((
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(serde_json::json!({ "error": e.to_string() })),
                )))
                .await?;
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
    tx: &mpsc::Sender<Result<User, (StatusCode, Json<serde_json::Value>)>>, user: UserModel,
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
