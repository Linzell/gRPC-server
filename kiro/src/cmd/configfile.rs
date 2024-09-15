use colored::*;
use std::io::{self, BufRead, Write};
use std::path::Path;

use crate::config::{self, Configuration};

pub fn run(config_dir: &Path, input: &mut dyn BufRead) -> Configuration {
    let conf = config::get();
    let mut new_conf = (*conf).clone();

    println!("\n{}", "🛠️  Configuration Editor".bright_cyan().bold());

    for field in Configuration::get_interactive_fields() {
        let current_value = (field.getter)(&new_conf);

        println!("\n{}", field.description.bright_green());
        println!("Current value: {}", current_value.yellow());
        println!("Possible values: {}", field.possible_values.cyan());
        print!(
            "{}",
            "Enter new value (or press Enter to keep current): ".bright_magenta()
        );
        io::stdout().flush().unwrap();

        let mut input_line = String::new();
        input.read_line(&mut input_line).unwrap();
        let new_value = input_line.trim();

        if !new_value.is_empty() {
            match (field.setter)(&mut new_conf, new_value) {
                Ok(()) => println!("{}", "✅ Updated successfully".green()),
                Err(e) => println!("{} {}", "❌ Error:".red(), e.to_string().red()),
            }
        }
    }

    config::save(&new_conf, config_dir).expect("Failed to save configuration");
    config::set(new_conf.clone());

    new_conf
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;
    use std::net::{IpAddr, Ipv6Addr, SocketAddr};
    use tempfile::TempDir;

    fn mock_run(input: &str) -> (Configuration, TempDir) {
        let temp_dir = TempDir::new().unwrap();
        let config_dir = temp_dir.path();
        let mut cursor = Cursor::new(input);
        let result = run(config_dir, &mut cursor);
        (result, temp_dir)
    }

    #[test]
    fn test_run_with_empty_input() {
        let input = "\n\n\n\n\n";
        let (result, _temp_dir) = mock_run(input);

        assert_eq!(result.logging.level, "INFO");
        assert_eq!(result.logging.json, false);
        assert_eq!(
            result.api.bind,
            SocketAddr::new(IpAddr::from(Ipv6Addr::UNSPECIFIED), 50051)
        );
        assert_eq!(result.api.secret, "");
    }

    #[test]
    fn test_run_with_valid_input() {
        let input = "DEBUG\ntrue\n127.0.0.1:8080\nnewsecret\n";
        let (result, _temp_dir) = mock_run(input);

        assert_eq!(result.logging.level, "DEBUG");
        assert_eq!(result.logging.json, true);
        assert_eq!(result.api.bind.to_string(), "127.0.0.1:8080");
        assert_eq!(result.api.secret, "newsecret");

        let global_config = config::get();
        assert_eq!(global_config.logging.level, "DEBUG");
        assert_eq!(global_config.logging.json, true);
        assert_eq!(global_config.api.bind.to_string(), "127.0.0.1:8080");
        assert_eq!(global_config.api.secret, "newsecret");
    }

    #[test]
    fn test_run_with_invalid_input() {
        let input = "INVALID\nnotboolean\ninvalid:port\n";
        let (result, _temp_dir) = mock_run(input);

        assert_eq!(result.logging.level, "INFO");
        assert_eq!(result.logging.json, false);
        assert_eq!(
            result.api.bind,
            SocketAddr::new(IpAddr::from(Ipv6Addr::UNSPECIFIED), 50051)
        );
    }
}
