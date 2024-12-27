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
mod models;
#[cfg(any(feature = "client", feature = "group"))]
mod utils;

/// # Link Model
///
/// The link model is a model that represents a link.
pub use models::{CreateLinkModel, LinkModel, LinkType};

/// # Mailer utils
///
/// The mailer utils module provides utilities for mailing.
#[cfg(any(feature = "client", feature = "group"))]
pub use utils::{Mailer, MailerTrait};

/// # Error
///
/// The error module provides error handling utilities.
pub use error::MailerError;

/// # Mock Mailer Trait
///
/// The mock mailer trait provides a mock trait for testing.
#[cfg(any(test, feature = "mock"))]
#[cfg(any(feature = "client", feature = "group"))]
pub use utils::MockMailerTrait;

/// # Mailer Trait
///
/// The mailer trait provides a trait for sending emails.
pub use lettre::{
    message::{header::ContentType, Message},
    transport::smtp::response::{Category, Code, Detail, Response, Severity},
};
