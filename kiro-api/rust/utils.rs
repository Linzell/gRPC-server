// utils.rs
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

use std::path::{Path, PathBuf};

/// Gets the output directory path from the OUT_DIR environment variable
///
/// # Returns
/// - `Ok(PathBuf)` - The output directory path
/// - `Err(Box<dyn Error>)` - If the OUT_DIR environment variable is not set or invalid
pub fn get_out_dir() -> Result<PathBuf, Box<dyn std::error::Error>> {
    Ok(PathBuf::from(std::env::var("OUT_DIR")?))
}

/// Gets the proto directory path by appending "proto" to CARGO_MANIFEST_DIR
///
/// # Returns
/// - `Ok(PathBuf)` - The proto directory path
/// - `Err(Box<dyn Error>)` - If CARGO_MANIFEST_DIR is not set or invalid
pub fn get_proto_dir() -> Result<PathBuf, Box<dyn std::error::Error>> {
    let proto_dir = PathBuf::from(std::env::var("CARGO_MANIFEST_DIR")?);
    Ok(proto_dir.join("proto"))
}

/// Creates a set of output directories for proto generation
///
/// # Arguments
/// * `out_dir` - Base output directory path
///
/// # Returns
/// - `Ok(())` - If directories were created successfully
/// - `Err(std::io::Error)` - If directory creation failed
pub fn create_output_directories(out_dir: &Path) -> Result<(), std::io::Error> {
    let dirs = ["google", "auth", "client", "common", "group", "project"];
    for dir in dirs {
        std::fs::create_dir_all(out_dir.join(dir))?;
    }
    Ok(())
}

/// Builds JSON support for protocol buffers
///
/// # Arguments
/// * `out_dir` - Output directory path
/// * `package` - Package name
/// * `paths` - Array of proto file paths
///
/// # Returns
/// - `Ok(())` - If JSON support was built successfully
/// - `Err(Box<dyn Error>)` - If building JSON support failed
#[cfg(feature = "json")]
pub fn build_json_support(
    out_dir: &Path, package: &str, paths: &[&str],
) -> Result<(), Box<dyn std::error::Error>> {
    let descriptor_set = std::fs::read(out_dir.join(package).join("proto_descriptor_v1.bin"))?;
    pbjson_build::Builder::new()
        .register_descriptors(&descriptor_set)?
        .ignore_unknown_fields()
        .out_dir(out_dir.join(package))
        .extern_path(".common", "crate::common")
        .build(paths)?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;
    use tempfile::TempDir;

    #[test]
    fn test_get_out_dir() {
        env::set_var("OUT_DIR", "/tmp/test");
        let result = get_out_dir();
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), PathBuf::from("/tmp/test"));
    }

    #[test]
    fn test_get_proto_dir() {
        env::set_var("CARGO_MANIFEST_DIR", "/tmp/project");
        let result = get_proto_dir();
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), PathBuf::from("/tmp/project/proto"));
    }

    #[test]
    fn test_create_output_directories() {
        let temp_dir = TempDir::new().unwrap();
        let result = create_output_directories(temp_dir.path());
        assert!(result.is_ok());

        // Verify directories were created
        let expected_dirs = ["google", "auth", "client", "common", "group", "project"];
        for dir in expected_dirs {
            assert!(temp_dir.path().join(dir).exists());
        }
    }
}
