// services/user/update_theme.rs
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

/// Updates a user's theme preference
///
/// # Arguments
///
/// * `service` - The UserService instance
/// * `request` - The request containing the new theme
///
/// # Returns
///
/// Returns an empty response on success
///
/// # Errors
///
/// Returns Status::unauthenticated if no valid session is found
/// Returns Status::invalid_argument if no theme is provided
/// Returns Status::internal for database errors
///
/// # Example
///
/// ```rust
/// let request = Request::new(UpdateUserThemeRequest {
///    theme: "dark".to_string()
/// });
/// let response = update_theme(&service, request).await?;
/// ```
pub async fn update_theme(
    service: &ClientService, request: Request<UpdateThemeRequest>,
) -> Result<Response<Empty>, Status> {
    // Get authenticated session from request extensions
    let session = request
        .extensions()
        .get::<SessionModel>()
        .ok_or_else(|| Status::unauthenticated("No valid session found"))?;

    // Get the new theme from the request
    let theme = request.get_ref().theme;

    // Update the user's theme in the database
    service
        .db
        .update_field(session.user_id.clone(), "theme", theme)
        .await
        .map_err(|e| Status::internal(e.to_string()))?;

    Ok(Response::new(Empty {}))
}
