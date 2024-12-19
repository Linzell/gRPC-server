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

mod builder;
mod config;
mod utils;

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
    // #[cfg(feature = "client")]
    // builder::build_client_service(&out_dir, &proto_dir, &proto_dir)?;
    // #[cfg(feature = "group")]
    // builder::build_group_service(&out_dir, &proto_dir, &proto_dir)?;

    Ok(())
}
