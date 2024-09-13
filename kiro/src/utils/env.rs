// utils/env.rs

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

// /// # Get env unsafe
// ///
// /// The `get_env_unsafe` method returns the value of an environment variable.
// ///
// /// **<!> Note: The method panics if the variable does not exist.**
// ///
// /// ```rust
// /// let value = get_env_unsafe("KEY");
// ///
// /// println!("🔑 Value: {:?}", value);
// /// ```
// pub fn get_env_unsafe(key: &str) -> String {
//     let envv = get_envv();
//     if envv.contains_key(key) {
//         envv.get(key).unwrap().clone()
//     } else {
//         panic!("Enviroment variable {0} is not set", key)
//     }
// }

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_env_or() {
        let key = "KEY";
        let value = "VALUE";
        let default = "DEFAULT";

        env::set_var(key, value);

        assert_eq!(get_env_or(key, default), value);
        assert_eq!(get_env_or("NOT_SET", default), default);
    }

    #[test]
    fn test_get_env_or_fail() {
        let key = "NONEXISTENT_KEY";
        let default = "DEFAULT";

        // Remove the key if it exists (just to be sure)
        env::remove_var(key);

        assert_eq!(get_env_or(key, default), default);
        assert_ne!(get_env_or(key, default), "SOME_OTHER_VALUE");
    }
}
