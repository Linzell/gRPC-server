// http/user/update_avatar.rs
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
    extract::{Multipart, State},
    http::StatusCode,
    response::IntoResponse,
    Extension, Json,
};
use kiro_database::db_bridge::DatabaseOperations;
use kiro_storage::{BucketS3, ByteStream};

use crate::SessionModel;

/// Avatar upload route handler webhook
///
/// # Description
/// Uploads a new avatar image for the current user
///
/// # Arguments
/// * `service` - The client service instance
/// * `session` - The current session model
/// * `multipart` - The multipart form data containing the avatar file
///
/// # Returns
/// * HTTP response with either:
///   * `200 OK` with the URL of the uploaded avatar image
///   * Error status code with message
///
/// # Errors
/// * `400 BAD REQUEST` - No avatar file provided
/// * `500 INTERNAL SERVER ERROR` - S3 storage or database error
///
/// # Example
/// ```rust,no_run
/// use axum::{Extension, extract::State, Json};
/// use http::HeaderMap;
/// use kiro_client::{ClientService, upload_avatar::upload_avatar, SessionModel};
/// use kiro_database::db_bridge::{Database, MockDatabaseOperations};
///
/// // Mock database
/// let mock_db = MockDatabaseOperations::new();
///
/// // Mock service
/// let service = ClientService {
///     db: Database::Mock(mock_db),
/// };
///
/// // Empty headers
/// let headers = HeaderMap::new();
///
/// // Mock session
/// let session = SessionModel::default();
///
/// // Mock multipart form data
/// let multipart = Multipart::new(headers, "boundary".to_string(), Vec::new());
///
/// // Async block to allow `await`
/// tokio::runtime::Runtime::new().unwrap().block_on(async {
/// let response = upload_avatar((State(service), Extension(session), multipart).await;
///
///     println!("Avatar uploaded");
/// });
/// ```
#[utoipa::path(
    post,
    path = "/user/upload_avatar",
    tag = "user",
    params(
        UpdateAvatarRequest
    ),
    responses(
        (status = 200, description = "Avatar uploaded", body = String),
        (status = 400, description = "No avatar file provided", body = String),
        (status = 500, description = "Internal server error", body = String)

    )
)]
pub async fn upload_avatar(
    State(service): State<ClientService>, Extension(session): Extension<SessionModel>,
    mut multipart: Multipart,
) -> impl IntoResponse {
    // Get the avatar file from the multipart form
    let mut file_data = Vec::new();

    while let Some(mut field) = multipart.next_field().await.unwrap_or(None) {
        if field.name().unwrap_or("") == "file" {
            while let Some(chunk) = field.chunk().await.unwrap_or(None) {
                file_data.extend_from_slice(&chunk);
            }
        }
    }

    if file_data.is_empty() {
        return (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({ "error": "No avatar file provided" })),
        );
    }

    // Upload file to S3
    let avatar_stream = ByteStream::from(file_data);
    let avatar_link = match BucketS3::new()
        .await
        .put_object(
            avatar_stream,
            session.user_id.clone(),
            "images",
            &format!("image-{}", session.user_id.clone()),
        )
        .await
    {
        Ok(link) => link,
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({ "error": e.to_string() })),
            )
        }
    };

    match service
        .db
        .update_field(session.user_id.clone(), "/avatar", avatar_link.clone())
        .await
    {
        Ok(_) => (
            StatusCode::OK,
            Json(serde_json::json!({ "url": avatar_link })),
        ),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": e.to_string() })),
        ),
    }
}
