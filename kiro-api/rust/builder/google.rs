// proto/google.rs
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

use crate::{config::BuildConfig, utils::build_json_support};
use std::path::Path;

pub fn build_google_protos(
    out_dir: &Path, proto_dir: &Path,
) -> Result<(), Box<dyn std::error::Error>> {
    let config = BuildConfig::default();
    let builder = tonic_build::configure()
        .out_dir(out_dir.join("google"))
        .protoc_arg("--experimental_allow_proto3_optional")
        .file_descriptor_set_path(out_dir.join("google").join("proto_descriptor_v1.bin"))
        .compile_well_known_types(config.compile_well_known_types)
        .type_attribute(".", "#[derive(utoipa::ToSchema)]");

    builder.compile_protos(
        &[
            proto_dir
                .join("google/google/protobuf")
                .join("any.proto")
                .to_str()
                .unwrap(),
            proto_dir
                .join("google/google/protobuf")
                .join("descriptor.proto")
                .to_str()
                .unwrap(),
            proto_dir
                .join("google/google/protobuf")
                .join("empty.proto")
                .to_str()
                .unwrap(),
            proto_dir
                .join("google/google/protobuf")
                .join("struct.proto")
                .to_str()
                .unwrap(),
            proto_dir
                .join("google/google/protobuf")
                .join("timestamp.proto")
                .to_str()
                .unwrap(),
        ],
        &[
            proto_dir.to_str().unwrap(),
            proto_dir.join("google").to_str().unwrap(),
        ],
    )?;

    #[cfg(feature = "json")]
    build_json_support(out_dir, "google", &[".google.protobuf"])?;

    Ok(())
}
