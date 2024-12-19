// builder/services/mod.rs
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

#[cfg(feature = "auth")]
mod auth;
#[cfg(feature = "auth")]
pub use auth::build_auth_service;

// #[cfg(feature = "group")]
// mod group;
// #[cfg(feature = "group")]
// pub use group::build_group_service;

// #[cfg(feature = "project")]
// mod project;
// #[cfg(feature = "project")]
// pub use project::build_project_service;
