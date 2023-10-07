// src/notify.rs
use crate::utils::HnComments;
use crate::hnscraper::{hnscraper, Comment}; // Import hnscraper module and Comment struct
use chrono::Utc;
use reqwest;
use select::document::Document;
use select::node::Node;
use select::predicate::Name;
use std::fs;
use std::vec::Vec;
use crate::utils::Config;
use dirs;
use std::time::Duration;
use tokio::time::sleep;
use crate::comments_checker::check_for_new_comments;

async fn notify_daemon(config: Config) {
    // Create local variables for configuration values
    let hn_username = config.hn_username;
    let db_username = config.db_username;
    let db_pass = config.db_pass;
    let port = config.port;
    let polling_interval_seconds = config.polling_interval_seconds;

    // Loop to continuously check for new comments
    loop {
        // Call hnscraper function to fetch comments
        match hnscraper(&hn_username) {
            Ok(comments) => {
                // Filter comments by hn_user, filters out our username's comments
                let filtered_comments = filter_comments(&comments, &hn_username);

                // Call the function to check for new comments and insert them into the database
                if let Err(err) = check_for_new_comments(filtered_comments.as_slice(), &db_username, &db_pass, port) {
                    eprintln!("Error inserting comments into the database: {}", err);
                }

            }
            Err(err) => {
                eprintln!("Error fetching comments: {}", err);
            }
        }

        // Sleep for the specified interval before checking again
        sleep(Duration::from_secs(polling_interval_seconds as u64)).await;
    }
}

pub async fn notify() {
    // Read configuration from ~/.hnotifyrc
    let config = read_config().unwrap_or_else(|| {
        println!("Failed to read configuration from ~/.hnotifyrc. Please check the config file.");
        std::process::exit(1);
    });

    // Call notify_daemon with the configuration
    notify_daemon(config).await;
}

// Change the return type of filter_comments to Vec<Comment>
fn filter_comments(comments: &[Comment], hn_username: &str) -> Vec<Comment> {
    let filtered_comments: Vec<Comment> = comments
        .iter()
        .filter(|comment| comment.hnuser != hn_username)
        .cloned() // Clone the Comment objects to create a new Vec<Comment>
        .collect();

    filtered_comments
}
fn read_config() -> Option<Config> {
    let home_dir = dirs::home_dir()?;
    let config_path = home_dir.join(".hnotifyrc");

    if config_path.exists() {
        if let Ok(contents) = fs::read_to_string(&config_path) {
            if let Ok(config) = serde_json::from_str::<Config>(&contents) {
                return Some(config);
            }
        }
    }

    None
}
