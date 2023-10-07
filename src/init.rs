// src/init.rs

//use mysql::prelude::Queryable;
use mysql::{Pool};
use std::path::PathBuf;
use std::fs;
use std::io::{stdin, stdout, Write};
use mysql::*;
use mysql::Opts;
use serde_json::{Value};
use crate::utils::HnComments; // Import the Hn_comments struct
use crate::utils::Config;

pub fn init(config_path: &PathBuf) {
    // Check if the configuration file already exists
    if config_path.exists() {
        println!("Configuration file already exists at {:?}", config_path);
    }

    // Create a new configuration with user input
    let existing_config = read_existing_config(&config_path);
    let config = get_config_from_user(existing_config);

    // Serialize the configuration to JSON
    let config_json = serde_json::to_string(&config).expect("Failed to serialize config");

    // Write the JSON to the configuration file
    fs::write(&config_path, config_json).expect("Failed to write config file");

    println!("Configuration initialized and saved to {:?}", config_path);

    // Establish a connection to the MySQL database without specifying a database
    let pool = create_mysql_pool(&config).expect("Failed to create MySQL pool");

    // Create the HnComments table and set the database in the connection URL
    create_hn_comments_table(&pool);

}

fn create_mysql_pool(config: &Config) -> Result<Pool> {
    let opts = Opts::from_url(&format!(
        "mysql://{}:{}@localhost:{}/{}",
        config.db_username, config.db_pass, config.port, "hnotify"
    ))?;

    let opts_without_db = Opts::from_url(&format!(
        "mysql://{}:{}@localhost:{}",
        config.db_username, config.db_pass, config.port
    ))?;

    // Attempt to create a connection pool with the specified database ("hnotify")
    // and fall back to creating one without a database if it fails
    match Pool::new(opts.clone()) {
        Ok(pool) => Ok(pool),
        Err(_) => {
            // Create the "hnotify" database if it doesn't exist
            let pool_without_db = Pool::new(opts_without_db)?;
            create_hnotify_database(&pool_without_db);

            // Close the connection pool without a database
            drop(pool_without_db);

            // Reconnect with the "hnotify" database
            let pool_with_db = Pool::new(opts)?;
            Ok(pool_with_db)
        }
    }
}

fn create_hn_comments_table(pool: &Pool) {
    // Build the CREATE TABLE statement based on the fields of the HnComments struct
    let create_table_query = build_create_table_query::<HnComments>("hn_comments");
    
    // Execute the CREATE TABLE query
    pool.prep_exec(create_table_query, ()).expect("Failed to create Hn_comments table");
}

fn build_create_table_query<T>(table_name: &str) -> String
where
    T: serde::Serialize + Default,
{
    let default_json = serde_json::to_value(&T::default()).expect("Failed to serialize default instance");
    let mut create_table_query = format!("CREATE TABLE IF NOT EXISTS {} (\n", table_name);

    // Add an auto-incrementing 'id' field as the first field
    create_table_query.push_str("    id INT AUTO_INCREMENT PRIMARY KEY,\n");

    if let Value::Object(fields) = default_json {
        let mut is_first_field = true;

        for (field, value) in fields {
            let field_type = match value {
                Value::String(_) => "VARCHAR(255)",
                Value::Number(_) => "INT",
                Value::Bool(_) => "BOOLEAN",
                _ => "VARCHAR(255)",
            };

            if is_first_field {
                is_first_field = false;
            } else {
                create_table_query.push_str(",\n");
            }

            create_table_query.push_str(&format!("    {} {}", field, field_type));
        }
    }

    create_table_query.push_str("\n)");

    create_table_query
}

fn create_hnotify_database(pool: &Pool) {
    // Create the "hnotify" database if it doesn't exist
    let create_db_query = r"CREATE DATABASE IF NOT EXISTS hnotify";
    pool.prep_exec(create_db_query, ()).expect("Failed to create hnotify database");
}

fn read_existing_config(config_path: &PathBuf) -> Option<Config> {
    // Read the existing configuration from the file, if it exists
    if let Ok(config_data) = fs::read_to_string(config_path) {
        if let Ok(existing_config) = serde_json::from_str(&config_data) {
            return Some(existing_config);
        }
    }
    None
}

fn get_config_from_user(existing_config: Option<Config>) -> Config {
    let mut config = existing_config.unwrap_or_default();

    // Helper function to get input with a default value
    fn get_input_with_default(prompt: &str, default: &str) -> String {
        let mut input = String::new();
        print!("{} [default is {}]: ", prompt, default);
        stdout().flush().expect("Failed to flush stdout");
        stdin().read_line(&mut input).expect("Failed to read user input");
        input.trim().is_empty().then(|| default.to_string()).unwrap_or(input.trim().to_string())
    }

    config.hn_username = get_input_with_default("Enter Hacker News username", &config.hn_username);
    config.db_username = get_input_with_default("Enter database username", &config.db_username);
    config.db_pass = get_input_with_default("Enter database password", &config.db_pass);

    let port_default = config.port.to_string();
    let port_input = get_input_with_default("Enter port (default is 3306)", &port_default);
    config.port = port_input.parse().unwrap_or(3306);

    // Ask the user for the polling interval in seconds
    let polling_default = config.polling_interval_seconds.to_string();
    let polling_input = get_input_with_default("Enter polling interval in seconds (default is 60)", &polling_default);
    config.polling_interval_seconds = polling_input.parse().unwrap_or(60);

    config
}
