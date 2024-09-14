use handlebars::{no_escape, Handlebars};
use std::io::{self, Write};
use std::path::Path;
use colored::*;

use crate::config::{self, Configuration};

pub fn run(config_dir: &Path) {
    let conf = config::get();
    let mut new_conf = (*conf).clone();

    println!("\n{}", "🛠️  Configuration Editor".bright_cyan().bold());

    for field in Configuration::get_interactive_fields() {
        let current_value = (field.getter)(&new_conf);

        println!("\n{}", field.description.bright_green());
        println!("Current value: {}", current_value.yellow());
        println!("Possible values: {}", field.possible_values.cyan());
        print!("{}", "Enter new value (or press Enter to keep current): ".bright_magenta());
        io::stdout().flush().unwrap();

        let mut input = String::new();
        io::stdin().read_line(&mut input).unwrap();
        let new_value = input.trim();

        if !new_value.is_empty() {
            match (field.setter)(&mut new_conf, new_value) {
                Ok(()) => println!("{}", "✅ Updated successfully".green()),
                Err(e) => println!("{} {}", "❌ Error:".red(), e.to_string().red()),
            }
        }
    }

    config::save(&new_conf, config_dir).expect("Failed to save configuration");
    config::set(new_conf.clone());

    let template = r#"
# Logging configuration
[logging]

level="{{ logging.level }}"
json={{ logging.json }}

# API interface configuration
[api]

bind="{{ api.bind }}"
secret="{{ api.secret }}"

# Gateway configuration
[gateway]

ca_cert="{{ gateway.ca_cert }}"
ca_key="{{ gateway.ca_key }}"

# Monitoring configuration
[monitoring]

bind="{{ monitoring.bind }}"

# User authentication configuration
[user_authentication]

enabled="{{ user_authentication.enabled }}"

# Backend interfaces configuration
[backend_interfaces]

bind="{{ backend_interfaces.bind }}"
"#;

    let mut reg = Handlebars::new();
    reg.register_escape_fn(no_escape);
    println!("\n{}", "🆕 New configuration:".bright_blue().bold());
    println!(
        "{}",
        reg.render_template(template, &new_conf)
            .expect("render configfile error")
            .cyan()
    );
}
