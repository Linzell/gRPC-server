// lib.rs
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

mod error;
mod http;
mod models;
mod services;
mod utils;

/// # Session Models
///
/// The session module provides models for authentication.
pub use models::{CreateSessionModel, SessionModel};

/// # User Models
///
/// The user module provides models for users.
pub use models::{
    CreateUserModel, Language, NotificationSettings, PrivacySettings, SecuritySettings, Theme,
    UserModel, UserSettings,
};

/// # Auth Services
///
/// The auth module provides services for authentication.
pub use services::AuthService;

/// # User Services
///
/// The user module provides services for users.
pub use services::ClientService;

/// # Auth HTTP1 Routes
///
/// The auth module provides HTTP1 routes for the authentication service.
pub use http::{auth_routes, login, logout, register};

/// # User HTTP1 Routes
///
/// The user module provides HTTP1 routes for the user service.
pub use http::{
    delete_user, disable_user, read_user, update_email, update_language, update_notifications,
    update_password, update_privacy, update_security, update_theme, user_routes,
};

#[cfg(feature = "mailer")]
pub use http::{send_email_to_change_email, send_email_to_change_password};

#[cfg(feature = "storage")]
pub use http::upload_avatar;

/// # Auth Server Builder
///
/// The auth module provides a builder for the authentication server.
pub use kiro_api::auth::v1::auth_service_server::AuthServiceServer;

/// # Client Server Builder
///
/// The client module provides a builder for the client server.
pub use kiro_api::client::v1::client_service_server::ClientServiceServer;

/// # Auth File Descriptor Set
///
/// The auth module provides the file descriptor set for the authentication service.
pub use kiro_api::auth::AUTH_V1_FILE_DESCRIPTOR_SET;

/// # User File Descriptor Set
///
/// The user module provides the file descriptor set for the user service.
pub use kiro_api::client::CLIENT_V1_FILE_DESCRIPTOR_SET;
