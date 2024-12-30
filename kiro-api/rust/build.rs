// build.rs
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

//! Build script for generating protocol buffer code
//!
//! This build script handles the compilation of protocol buffer definitions into Rust code.
//! It supports building common protos, Google API protos, and service-specific protos based
//! on enabled feature flags.

mod builder;
mod config;
mod utils;

/// Main build function that orchestrates the protocol buffer compilation process
///
/// # Returns
///
/// Returns `Ok(())` if compilation succeeds, or an error if any step fails
///
/// # Errors
///
/// Will return an error if:
/// - Output/proto directories cannot be accessed
/// - Directory creation fails
/// - Proto compilation fails for any service
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let out_dir = utils::get_out_dir()?;
    let proto_dir = utils::get_proto_dir()?;

    // Create necessary directories
    utils::create_output_directories(&out_dir)?;

    // Build common protos
    #[cfg(feature = "api")]
    builder::build_common_protos(&out_dir, &proto_dir, &proto_dir)?;

    // Build Google protos
    #[cfg(feature = "api")]
    builder::build_google_protos(&out_dir, &proto_dir)?;

    // Build service protos
    #[cfg(feature = "auth")]
    builder::build_auth_service(&out_dir, &proto_dir, &proto_dir)?;
    #[cfg(feature = "auth")]
    builder::build_client_service(&out_dir, &proto_dir, &proto_dir)?;
    // #[cfg(feature = "group")]
    // builder::build_group_service(&out_dir, &proto_dir, &proto_dir)?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_output_directory_creation() {
        let test_dir = PathBuf::from("target/test_out");
        utils::create_output_directories(&test_dir).unwrap();
        assert!(test_dir.exists());
    }

    #[test]
    fn test_proto_directory_exists() {
        let proto_dir = utils::get_proto_dir().unwrap();
        assert!(proto_dir.exists());
    }

    #[test]
    fn test_out_directory_exists() {
        let out_dir = utils::get_out_dir().unwrap();
        assert!(out_dir.exists());
    }
}
