// utils/mod.rs
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

/// # Documentation
///
/// The utils module provides a collection of utilities for working with various parts of the application.
#[cfg(feature = "documentation")]
pub mod doc;

/// # gRPC Utilities
///
/// The gRPC utilities module provides helper functions for working with gRPC services.
#[cfg(feature = "client")]
pub mod grpc_utils;

/// # Telemetry
///
/// The telemetry module provides distributed tracing functionality.
#[cfg(feature = "tracing")]
pub mod telemetry;

/// # Error Mailer
///
/// The error_mailer module handles sending error notifications via email.
#[cfg(feature = "mailer")]
pub mod error_mailer;
