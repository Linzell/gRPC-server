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

//! Fix generated file error, with clippy: "the following explicit lifetimes could be elided".
//! Reason: I build generated files with `prost-build` and I don't want to modify them manually
//! to remove the lifetimes.
//! This is a workaround to fix the clippy error.
#![allow(clippy::needless_lifetimes)]

pub use prost;

#[cfg(feature = "api")]
mod api;

#[cfg(feature = "auth")]
pub use api::auth;
#[cfg(any(feature = "auth", feature = "client"))]
pub use api::client;
#[cfg(feature = "api")]
pub use api::common;
#[cfg(feature = "api")]
pub use api::google;
// #[cfg(feature = "group")]
// pub use api::group;
