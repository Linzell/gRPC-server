// services/user/update_avatar.rs
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
use kiro_storage::{BucketS3, ByteStream};
use tonic::{Request, Response, Status};

use crate::SessionModel;

/// Updates a user's avatar image
///
/// # Arguments
///
/// * `service` - The UserService instance
/// * `request` - The request containing the avatar file
///
/// # Returns
///
/// Returns the URL of the uploaded avatar image on success
///
/// # Errors
///
/// Returns Status::unauthenticated if no valid session is found
/// Returns Status::invalid_argument if no avatar file is provided
/// Returns Status::internal for S3 storage or database errors
///
/// # Example
///
/// ```rust, ignore
/// let request = Request::new(UpdateUserAvatarRequest {
///     avatar: Some(File {
///         name: "avatar.jpg".to_string(),
///         content: vec![...],
///         type: "image/jpeg".to_string()
///     })
/// });
/// let response = update_avatar(&service, request).await?;
/// println!("New avatar URL: {}", response.get_ref().avatar);
/// ```
pub async fn upload_avatar(
    service: &ClientService, request: Request<UploadAvatarRequest>,
) -> Result<Response<UploadAvatarResponse>, Status> {
    // Get authenticated session from request extensions
    let session = request
        .extensions()
        .get::<SessionModel>()
        .ok_or_else(|| Status::unauthenticated("No valid session found"))?;

    // Extract avatar file from request
    let file = request
        .get_ref()
        .file
        .clone()
        .ok_or_else(|| Status::invalid_argument("No avatar provided"))?;

    // Upload file to S3
    let avatar_stream = ByteStream::from(file.content);
    let avatar_link = BucketS3::new()
        .await
        .put_object(
            avatar_stream,
            session.user_id.clone(),
            "images",
            &format!("image-{}", session.user_id.clone()),
        )
        .await
        .map_err(|e| Status::internal(e.to_string()))?;

    // Update user's avatar field in database
    service
        .db
        .update_field(session.user_id.clone(), "/avatar", avatar_link.clone())
        .await
        .map_err(|e| Status::internal(e.to_string()))?;

    Ok(Response::new(UploadAvatarResponse { url: avatar_link }))
}
