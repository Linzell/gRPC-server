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

/// Configuration for protocol buffer code generation build process
///
/// This struct controls how protocol buffer code is generated, including settings
/// for well-known type handling.
#[allow(dead_code)]
pub struct BuildConfig {
    /// Path to import well-known protobuf type definitions from
    ///
    /// When the "json" feature is enabled, this defaults to "::pbjson_types"
    /// Otherwise it defaults to "::prost_types"
    pub well_known_types_path: &'static str,

    /// Whether to compile the well-known protobuf types
    ///
    /// When true, the well-known types will be compiled along with user-defined types.
    /// When false, they will be expected to be provided externally.
    pub compile_well_known_types: bool,
}

impl Default for BuildConfig {
    /// Creates a new BuildConfig with default settings
    ///
    /// # Returns
    /// A BuildConfig instance with:
    /// - well_known_types_path set based on json feature flag
    /// - compile_well_known_types set to true
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_config_default() {
        let config = BuildConfig::default();
        assert!(config.compile_well_known_types);

        #[cfg(feature = "json")]
        assert_eq!(config.well_known_types_path, "::pbjson_types");

        #[cfg(not(feature = "json"))]
        assert_eq!(config.well_known_types_path, "::prost_types");
    }
}
