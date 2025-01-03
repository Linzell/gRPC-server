// models/mod.rs
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

mod session_model;
mod user_model;

/// # Session Models
///
/// The session model provides models for authentication.
pub use session_model::{CreateSessionModel, SessionModel};

/// # User Models
///
/// The user model provides models for users.
pub use user_model::{
    CreateUserModel, Language, NotificationSettings, PrivacySettings, SecuritySettings, Theme,
    UserModel, UserSettings,
};
