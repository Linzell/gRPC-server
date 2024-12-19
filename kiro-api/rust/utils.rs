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

pub fn get_out_dir() -> Result<PathBuf, Box<dyn std::error::Error>> {
    Ok(PathBuf::from(std::env::var("OUT_DIR")?))
}

pub fn get_proto_dir() -> Result<PathBuf, Box<dyn std::error::Error>> {
    let proto_dir = PathBuf::from(std::env::var("CARGO_MANIFEST_DIR")?);
    Ok(proto_dir.join("proto"))
}

pub fn create_output_directories(out_dir: &Path) -> Result<(), std::io::Error> {
    let dirs = ["google", "auth", "client", "common", "group", "project"];
    for dir in dirs {
        std::fs::create_dir_all(out_dir.join(dir))?;
    }
    Ok(())
}

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
