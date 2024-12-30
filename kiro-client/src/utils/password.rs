// utils/password.rs
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

/// # PasswordError
///
/// The PasswordError enum is an enum that represents the errors for passwords.
#[derive(thiserror::Error, Debug, PartialEq)]
pub enum PasswordError {
    #[error("Password too short. Minimum size: {} characters", PASS_MIN_LENGTH)]
    TooShort,

    #[error("Password too long. Maximum size: {} characters", PASS_MAX_LENGTH)]
    TooLong,

    #[error(
        "Password doesn't contain enough Special characters. Minimum special characters: {}",
        PASS_MIN_SYMBOLS
    )]
    NotEnoughSymbols,

    #[error(
        "Password doesn't contain enough Digits. Minimum digits: {}",
        PASS_MIN_DIGITS
    )]
    NotEnoughDigits,

    #[error(
        "Password doesn't contain enough Letters. Minimum letters: {}",
        PASS_MIN_LETTERS
    )]
    NotEnoughLetters,
}

const PASS_MIN_LENGTH: usize = 8;
const PASS_MAX_LENGTH: usize = 160;

const PASS_MIN_SYMBOLS: usize = 1;
const PASS_MIN_DIGITS: usize = 1;
const PASS_MIN_LETTERS: usize = 1;

const DIGITS: &str = "1234567890";
const LETTERS: &str = "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz";
const SPECIAL_SYMBOLS: &str = "-_/\\(){}[]|!@#$%^&*)+=\"\';:<>,.?";

/// # Contains number
///
/// The `contains_number` method returns the number of characters in a password.
fn contains_number(pass: &str, allowed: &str) -> usize {
    let mut i: usize = 0;
    for character in pass.chars() {
        if allowed.contains(character) {
            i += 1;
        }
    }
    i
}

/// # Valid password
///
/// The `valid_password` method returns a result if the password is valid.
pub fn valid_password(password: &str) -> Result<(), PasswordError> {
    if password.len() < PASS_MIN_LENGTH {
        return Err(PasswordError::TooShort);
    }
    if password.len() > PASS_MAX_LENGTH {
        return Err(PasswordError::TooLong);
    }
    if contains_number(password, SPECIAL_SYMBOLS) < PASS_MIN_SYMBOLS {
        return Err(PasswordError::NotEnoughSymbols);
    }
    if contains_number(password, DIGITS) < PASS_MIN_DIGITS {
        return Err(PasswordError::NotEnoughDigits);
    }
    if contains_number(password, LETTERS) < PASS_MIN_LETTERS {
        return Err(PasswordError::NotEnoughLetters);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_contains_number_digits() {
        let number = contains_number("123", DIGITS);
        assert_eq!(number, 3);
    }

    #[test]
    fn test_contains_number_letters() {
        let number = contains_number("abc", LETTERS);
        assert_eq!(number, 3);
    }

    #[test]
    fn test_contains_number_symbols() {
        let number = contains_number("!@#", SPECIAL_SYMBOLS);
        assert_eq!(number, 3);
    }

    #[test]
    fn test_valid_password() {
        let result = valid_password("123abc!@#");
        assert!(result.is_ok());
    }

    #[test]
    fn test_valid_password_fail() {
        let result = valid_password("123abc");
        assert!(result.is_err());
    }

    #[test]
    fn test_valid_password_too_short() {
        let result = valid_password("123");
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), PasswordError::TooShort);
    }

    #[test]
    fn test_valid_password_too_long() {
        let result = valid_password(&"1234567890".repeat(17));
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), PasswordError::TooLong);
    }

    #[test]
    fn test_valid_password_not_enough_symbols() {
        let result = valid_password("123abc123abc");
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), PasswordError::NotEnoughSymbols);
    }

    #[test]
    fn test_valid_password_not_enough_digits() {
        let result = valid_password("abc!@#abc!@#");
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), PasswordError::NotEnoughDigits);
    }

    #[test]
    fn test_valid_password_not_enough_letters() {
        let result = valid_password("123!@#123!@#");
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), PasswordError::NotEnoughLetters);
    }
}
