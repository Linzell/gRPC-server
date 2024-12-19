// config.rs
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

#[allow(dead_code)]
pub struct BuildConfig {
    pub well_known_types_path: &'static str,
    pub compile_well_known_types: bool,
}

impl Default for BuildConfig {
    fn default() -> Self {
        Self {
            #[cfg(feature = "json")]
            well_known_types_path: "::pbjson_types",
            #[cfg(not(feature = "json"))]
            well_known_types_path: "::prost_types",
            compile_well_known_types: true,
        }
    }
}
