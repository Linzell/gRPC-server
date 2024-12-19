// proto/mod.rs
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

#[cfg(feature = "api")]
mod common;
#[cfg(feature = "api")]
mod google;
mod services;

#[cfg(feature = "api")]
pub use common::build_common_protos;

#[cfg(feature = "api")]
pub use google::build_google_protos;

#[cfg(feature = "auth")]
pub use services::build_auth_service;

// #[cfg(feature = "group")]
// pub use services::build_group_service;

// #[cfg(feature = "project")]
// pub use services::build_project_service;
