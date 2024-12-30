// group.rs
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

pub mod v1 {
    include!(concat!(env!("OUT_DIR"), "/group/group.v1.rs"));
    #[cfg(feature = "json")]
    include!(concat!(env!("OUT_DIR"), "/group/group.v1.serde.rs"));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_proto_generated() {
        // Verify the proto module is accessible
        assert!(std::fs::metadata(concat!(env!("OUT_DIR"), "/group/group.v1.rs")).is_ok());
    }

    #[cfg(feature = "json")]
    #[test]
    fn test_serde_generated() {
        // Verify the serde module is accessible when json feature enabled
        assert!(std::fs::metadata(concat!(env!("OUT_DIR"), "/group/group.v1.serde.rs")).is_ok());
    }
}
