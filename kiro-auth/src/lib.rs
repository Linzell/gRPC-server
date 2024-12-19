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

/// # Auth Module
///
/// The auth module provides authentication functionality.
pub use models::SessionStore;

/// # Auth Models
///
/// The auth module provides models for authentication.
pub use models::SessionModel;

/// # Auth Services
///
/// The auth module provides services for authentication.
pub use services::AuthService;

/// # Auth Server Builder
///
/// The auth module provides a builder for the authentication server.
pub use kiro_api::auth::v1::auth_service_server::AuthServiceServer;

/// # Auth File Descriptor Set
///
/// The auth module provides the file descriptor set for the authentication service.
pub use kiro_api::auth::AUTH_V1_FILE_DESCRIPTOR_SET;
