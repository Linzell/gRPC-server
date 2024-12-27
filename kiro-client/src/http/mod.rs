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
// mod user;

/// # Auth HTTP1 Routes
///
/// The auth module provides HTTP1 routes for the authentication service.
pub use auth::auth_routes;

// /// # User HTTP1 Routes
// ///
// /// The user module provides HTTP1 routes for the user service.
// pub use user::user_routes;
