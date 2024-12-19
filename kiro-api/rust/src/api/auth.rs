// auth.rs
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

pub const AUTH_V1_FILE_DESCRIPTOR_SET: &[u8] =
    include_bytes!(concat!(env!("OUT_DIR"), "/auth/proto_descriptor_v1.bin"));

pub mod v1 {
    include!(concat!(env!("OUT_DIR"), "/auth/auth.v1.rs"));
    #[cfg(feature = "json")]
    include!(concat!(env!("OUT_DIR"), "/auth/auth.v1.serde.rs"));
}