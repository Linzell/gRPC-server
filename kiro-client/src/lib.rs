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
#[cfg(any(feature = "auth", feature = "user"))]
mod http;
#[cfg(any(feature = "auth", feature = "user"))]
mod models;
#[cfg(any(feature = "auth", feature = "user"))]
mod services;
#[cfg(any(feature = "auth", feature = "user"))]
mod utils;

/// # Session Models
///
/// The session module provides models for authentication.
#[cfg(feature = "auth")]
pub use models::{CreateSessionModel, SessionModel};

/// # User Models
///
/// The user module provides models for users.
#[cfg(any(feature = "auth", feature = "user"))]
pub use models::{
    CreateUserModel, Language, NotificationSettings, PrivacySettings, SecuritySettings, Theme,
    UserModel, UserSettings,
};

/// # Auth Services
///
/// The auth module provides services for authentication.
#[cfg(feature = "auth")]
pub use services::AuthService;

/// # User Services
///
/// The user module provides services for users.
#[cfg(feature = "user")]
pub use services::ClientService;

/// # Auth HTTP1 Routes
///
/// The auth module provides HTTP1 routes for the authentication service.
#[cfg(feature = "auth")]
pub use http::auth_routes;

/// # Auth Server Builder
///
/// The auth module provides a builder for the authentication server.
#[cfg(feature = "auth")]
pub use kiro_api::auth::v1::auth_service_server::AuthServiceServer;

/// # Client Server Builder
///
/// The client module provides a builder for the client server.
#[cfg(feature = "user")]
pub use kiro_api::client::v1::client_service_server::ClientServiceServer;

/// # Auth File Descriptor Set
///
/// The auth module provides the file descriptor set for the authentication service.
#[cfg(feature = "auth")]
pub use kiro_api::auth::AUTH_V1_FILE_DESCRIPTOR_SET;

/// # User File Descriptor Set
///
/// The user module provides the file descriptor set for the user service.
#[cfg(feature = "user")]
pub use kiro_api::client::CLIENT_V1_FILE_DESCRIPTOR_SET;
