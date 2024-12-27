// src/server/reflection.rs
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

use tonic_reflection::server::v1alpha::{ServerReflection, ServerReflectionServer};

pub fn setup_reflection_service() -> ServerReflectionServer<impl ServerReflection> {
    let builder = tonic_reflection::server::Builder::configure();

    #[cfg(feature = "client")]
    let builder = builder
        .register_encoded_file_descriptor_set(kiro_client::CLIENT_V1_FILE_DESCRIPTOR_SET)
        .register_encoded_file_descriptor_set(kiro_client::AUTH_V1_FILE_DESCRIPTOR_SET);

    builder
        .register_encoded_file_descriptor_set(tonic_health::pb::FILE_DESCRIPTOR_SET)
        .build_v1alpha()
        .unwrap()
}
