// services/user/update_privacy.rs
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

/// Updates a user's privacy settings
///
/// # Arguments
///
/// * `service` - The UserService instance
/// * `request` - The request containing the new privacy settings
///
/// # Returns
///
/// Returns an empty response on success
///
/// # Errors
///
/// Returns Status::unauthenticated if no valid session is found
/// Returns Status::invalid_argument if no privacy settings are provided
/// Returns Status::internal for database errors
///
/// # Example
///
/// ```rust
/// let request = Request::new(UpdateUserPrivacyRequest {
///    field: "data_collection".to_string(),
///    value: true,
/// });
/// let response = update_privacy(&service, request).await?;
/// ```
pub async fn update_privacy(
    service: &ClientService, request: Request<UpdatePrivacyRequest>,
) -> Result<Response<Empty>, Status> {
    // Get authenticated session from request extensions
    let session = request
        .extensions()
        .get::<SessionModel>()
        .ok_or_else(|| Status::unauthenticated("No valid session found"))?;

    // Get the field and value from the request
    let field = request.get_ref().field.as_str();
    let value = request.get_ref().value;

    // Validate the privacy field name
    if !["data_collection", "location"].contains(&field) {
        return Err(Status::invalid_argument("Invalid privacy field"));
    }

    // Update the user's privacy settings in the database
    service
        .db
        .update_field(
            session.user_id.clone(),
            &format!("settings/privacy/{}", field),
            value,
        )
        .await
        .map_err(|e| Status::internal(e.to_string()))?;

    Ok(Response::new(Empty {}))
}
