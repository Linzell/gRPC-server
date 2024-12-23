// services/user/send_email_to_change_password.rs
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

use chrono::{Days, Utc};
use kiro_database::{db_bridge::DatabaseOperations, get_env_or};
use kiro_mailer::{ContentType, LinkModel, LinkType, Mailer, MailerTrait};
use tonic::{Request, Response, Status};

use crate::{models::UserModel, SessionModel};

/// Sends an email to the user to change their password
///
/// # Arguments
///
/// * `service` - The UserService instance providing database access
/// * `request` - The gRPC request containing the new password
///
/// # Returns
///
/// * `Ok(Response)` - Email sent successfully
/// * `Err(Status)` - Various error conditions with appropriate status codes
///
/// # Flow
///
/// 1. Validates session and password
/// 2. Generates a temporary change key
/// 3. Sends an email with a link to confirm the change
///
/// # Errors
///
/// * `UNAUTHENTICATED` - No valid session
/// * `INVALID_ARGUMENT` - No password provided
/// * `INTERNAL` - Database or email sending errors
pub async fn send_email_to_change_password(
    service: &ClientService, request: Request<Empty>,
) -> Result<Response<Empty>, Status> {
    // Get authenticated session from middleware
    let session = request
        .extensions()
        .get::<SessionModel>()
        .ok_or_else(|| Status::unauthenticated("No valid session found"))?;

    // Get user details
    let user = service
        .db
        .select::<UserModel>(session.user_id.clone())
        .await
        .map_err(|e| Status::internal(e.to_string()))?
        .ok_or_else(|| Status::not_found("User not found"))?;

    // Create change password link that expires in 24 hours
    let expiry_time = Utc::now()
        .checked_add_days(Days::new(1))
        .ok_or_else(|| Status::internal("Failed to calculate expiry time"))?;

    let link = LinkModel::create_from_user(
        &service.db,
        session.user_id.clone(),
        expiry_time,
        LinkType::PasswordChange,
    )
    .await
    .map_err(|e| Status::internal(format!("Failed to create change link: {}", e)))?
    .construct_link();

    // Load and populate email template
    let template = Mailer::load_template("password_change.html")
        .await
        .map_err(|e| Status::internal(format!("Failed to load email template: {}", e)))?
        .replace("${{USER_NAME}}", &user.email)
        .replace("${{CHANGE_URL}}", &link);

    let from = get_env_or("SMTP_USER", "test@example.com");

    // Build and send email
    let message = Mailer::build_mail(
        &from,
        &user.email,
        "Change Password",
        ContentType::TEXT_HTML,
        template,
    )
    .map_err(|e| Status::internal(format!("Failed to build email: {}", e)))?;

    Mailer::new()
        .send_mail(message)
        .await
        .map_err(|e| Status::internal(format!("Failed to send email: {}", e)))?;

    Ok(Response::new(Empty {}))
}
