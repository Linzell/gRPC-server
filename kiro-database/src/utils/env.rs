// utils/env.rs
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

use std::{collections::HashMap, env};

/// # Get envv
///
/// The `get_envv` method returns a HashMap of environment variables.
///
/// ```rust
/// let envv = get_envv();
///
/// println!("🔧 Enviroment variables: {
///  #envv
/// }");
/// ```
fn get_envv() -> HashMap<String, String> {
    env::vars().collect()
}

/// # Get env or
///
/// The `get_env_or` method returns either the value of an environment variable or the default provided.
///
/// ```rust
/// let value = get_env_or("KEY", "default");
///
/// println!("🔑 Value: {:?}", value);
/// ```
pub fn get_env_or(key: &str, default: &str) -> String {
    let envv = get_envv();
    if envv.contains_key(key) {
        // Safety: The HashMap is already checked for the key
        envv.get(key).unwrap().clone()
    } else {
        default.to_string()
    }
}

/// # Get env unsafe
///
/// The `get_env_unsafe` method returns the value of an environment variable.
///
/// **<!> Note: The method panics if the variable does not exist.**
///
/// ```rust
/// let value = get_env_unsafe("KEY");
///
/// println!("🔑 Value: {:?}", value);
/// ```
pub fn get_env_unsafe(key: &str) -> String {
    let envv = get_envv();
    if envv.contains_key(key) {
        envv.get(key).unwrap().clone()
    } else {
        panic!("Enviroment variable {0} is not set", key)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_envv() {
        let envv = get_envv();
        assert!(envv.contains_key("PATH"));
    }

    #[test]
    fn test_get_env_or() {
        let envv = get_env_or("PATH", "test");
        assert_eq!(envv, env::var("PATH").unwrap());
    }

    #[test]
    fn test_get_env_unsafe() {
        let envv = get_env_unsafe("PATH");
        assert_eq!(envv, env::var("PATH").unwrap());
    }
}
