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
#[cfg(feature = "services")]
mod http;
#[cfg(feature = "models")]
mod models;
#[cfg(feature = "services")]
mod services;
mod utils;

/// # Session Models
///
/// The session module provides models for authentication.
#[cfg(feature = "models")]
pub use models::{CreateSessionModel, SessionModel};

/// # Auth Services
///
/// The auth module provides services for authentication.
#[cfg(feature = "services")]
pub use services::AuthService;

/// # Auth HTTP1 Routes
///
/// The auth module provides HTTP1 routes for the authentication service.
#[cfg(feature = "services")]
pub use http::auth_routes;

/// # Auth Server Builder
///
/// The auth module provides a builder for the authentication server.
#[cfg(feature = "services")]
pub use kiro_api::auth::v1::auth_service_server::AuthServiceServer;

/// # Auth File Descriptor Set
///
/// The auth module provides the file descriptor set for the authentication service.
#[cfg(feature = "services")]
pub use kiro_api::auth::AUTH_V1_FILE_DESCRIPTOR_SET;
