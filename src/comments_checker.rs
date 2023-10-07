use ring::digest::{Context, SHA256};
use mysql::{Pool, prelude::*};
use crate::hnscraper::Comment;
use std::process::Command;


pub fn check_for_new_comments(
    filtered_comments: &[Comment],
    db_username: &str,
    db_pass: &str,
    port: i32,
) -> Result<Vec<Comment>, mysql::Error> {
    // Create a connection pool to the MySQL database
    let url = format!("mysql://{}:{}@localhost:{}/hnotify", db_username, db_pass, port);
    let pool = Pool::new(url)?;

    // Determine the number of existing hashes to retrieve based on the length of filtered_comments
    let num_hashes_to_retrieve = filtered_comments.len();

    // Query the database to retrieve existing SHA-256 hashes
    let existing_hashes: Vec<String> = pool
        .prep_exec(
            format!("SELECT hash_sha FROM hn_comments ORDER BY id DESC LIMIT {}", num_hashes_to_retrieve),
            (),
        )
        .map(|result| {
            result
                .map(|x| x.unwrap())
                .map(|row| mysql::from_row(row))
                .collect()
        })?;

    // Create a new vector to store the comments that don't exist in the database
    let mut new_comments: Vec<Comment> = Vec::new();

    // Iterate over the filtered comments
    for comment in filtered_comments {
        // Calculate the SHA-256 hash for the comment
        let hash = calculate_sha256_hash(&format!("{}", comment.commtext));

        // Check if the hash exists in the list of existing hashes
        if !existing_hashes.contains(&hash) {
            // If the hash doesn't exist, add the comment to the new vector
            new_comments.push(comment.clone());

            // Insert the hash into the database
            pool.prep_exec("INSERT INTO hn_comments (hash_sha) VALUES (?)", (hash,))?;
        }
    }

    send_system_notification(&new_comments);

    Ok(new_comments)
}

pub fn send_system_notification(new_comments: &[Comment]) {
    match new_comments.len() {
        0 => {
            // No new comments, do nothing
        }
        1 => {
            // One new comment, display user and first 10 words
            if let Some(comment) = new_comments.first() {
                let user_and_comment = format!("{}: {}", comment.hnuser, get_first_10_words(&comment.commtext));
                send_notification("HN, ", &user_and_comment);
            }
        }
        _ => {
            // Multiple new comments, display count and first 10 words of one comment
            let count = new_comments.len();
            if let Some(comment) = new_comments.first() {
                let message = format!("{} new comments: {}", count, get_first_10_words(&comment.commtext));
                send_notification("HN, ", &message);
            }
        }
    }
}

fn send_notification(title: &str, message: &str) {
    Command::new("notify-send")
        .arg(title)
        .arg(message)
        .spawn()
        .expect("Failed to execute notify-send command");
}

fn get_first_10_words(text: &str) -> String {
    text.split_whitespace().take(10).collect::<Vec<&str>>().join(" ")
}

// Function to calculate SHA-256 hash for a string
fn calculate_sha256_hash(input: &str) -> String {
    let mut context = Context::new(&SHA256);
    context.update(input.as_bytes());
    let digest = context.finish();

    let hash = digest.as_ref();
    hex::encode(hash)
}

