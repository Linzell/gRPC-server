// http/mod.rs
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

mod auth;
mod user;

/// # Auth HTTP1 Routes
///
/// The auth module provides HTTP1 routes for the authentication service.
pub use auth::{auth_routes, login, logout, register};

/// # User HTTP1 Routes
///
/// The user module provides HTTP1 routes for the user service.
pub use user::{
    delete_user, disable_user, read_user, update_email, update_language, update_notifications,
    update_password, update_privacy, update_security, update_theme, user_routes,
};

/// # User HTTP1 Routes (Mailer)
///
/// The user module provides HTTP1 routes for the user service with mailer support.
#[cfg(feature = "mailer")]
pub use user::{send_email_to_change_email, send_email_to_change_password};

/// # User HTTP1 Routes (Storage)
///
/// The user module provides HTTP1 routes for the user service with storage support.
#[cfg(feature = "storage")]
pub use user::upload_avatar;
