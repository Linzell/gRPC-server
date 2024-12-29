// proto/services/auth.rs
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

pub fn build_auth_service(
    out_dir: &Path, proto_dir: &Path, cs_dir: &Path,
) -> Result<(), Box<dyn std::error::Error>> {
    let config = BuildConfig::default();
    let builder = tonic_build::configure()
        .out_dir(out_dir.join("auth"))
        .protoc_arg("--experimental_allow_proto3_optional")
        .file_descriptor_set_path(out_dir.join("auth").join("proto_descriptor_v1.bin"))
        .compile_well_known_types(config.compile_well_known_types)
        .type_attribute(".", "#[derive(utoipa::ToSchema)]")
        .type_attribute(
            ".auth.v1.AuthRequest",
            r#"#[derive(utoipa::IntoParams)]
            #[into_params(parameter_in = Query)]"#,
        )
        .extern_path(".google.protobuf", "crate::google::protobuf");

    builder.compile(
        &[
            cs_dir
                .join("auth/v1")
                .join("session.proto")
                .to_str()
                .unwrap(),
            cs_dir
                .join("auth/v1")
                .join("auth_service.proto")
                .to_str()
                .unwrap(),
        ],
        &[
            proto_dir.to_str().unwrap(),
            proto_dir.join("google").to_str().unwrap(),
        ],
    )?;

    #[cfg(feature = "json")]
    build_json_support(out_dir, "auth", &[".auth"])?;

    Ok(())
}
