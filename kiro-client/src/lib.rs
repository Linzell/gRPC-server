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
#[cfg(feature = "models")]
mod models;

/// # User Models
///
/// The user module provides models for users.
#[cfg(feature = "models")]
pub use models::{
    CreateUserModel, Language, NotificationSettings, PrivacySettings, SecuritySettings, Theme,
    UserModel, UserSettings,
};

/// # Client Server Builder
///
/// The client module provides a builder for the client server.
#[cfg(feature = "services")]
pub use kiro_api::client::v1::client_service_server::ClientServiceServer;

/// # User File Descriptor Set
///
/// The user module provides the file descriptor set for the user service.
#[cfg(feature = "services")]
pub use kiro_api::client::CLIENT_V1_FILE_DESCRIPTOR_SET;
