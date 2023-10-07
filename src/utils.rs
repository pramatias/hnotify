// src/utils.rs

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use chrono::{NaiveDateTime, Utc}; // Import chrono for timestamp
use serde::ser::SerializeStruct;

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    pub hn_username: String,
    pub db_username: String,
    pub db_pass: String,
    pub port: i32,
    pub polling_interval_seconds: i32,
}

impl Default for Config {
    fn default() -> Self {
        // Default configuration values
        Config {
            hn_username: String::new(),
            db_username: String::new(),
            db_pass: String::new(),
            port: 3306, // Default port value
            polling_interval_seconds: 60, // Default polling interval
        }
    }
}


#[derive(Debug)]
pub struct HnComments {
    timestamp: NaiveDateTime,
    hash_sha: String,
}

impl HnComments {
    pub fn new(timestamp: NaiveDateTime, text: &str) -> Self {
        let hash_sha = HnComments::calculate_hash(text);
        Self { timestamp, hash_sha }
    }

    // Calculate SHA-256 hash
    fn calculate_hash(text: &str) -> String {
        let mut hasher = Sha256::new();
        hasher.update(text);
        format!("{:x}", hasher.finalize())
    }
}

impl Serialize for HnComments {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut state = serializer.serialize_struct("HnComments", 2)?;
        state.serialize_field("timestamp", &self.timestamp.to_string())?;
        state.serialize_field("hash_sha", &self.hash_sha)?;
        state.end()
    }
}

impl Default for HnComments {
    fn default() -> Self {
        HnComments {
            timestamp: Utc::now().naive_utc(), // Default to the current UTC time
            hash_sha: String::new(),
        }
    }
}
