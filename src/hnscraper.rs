//hnscraper.rs

extern crate scraper;
use scraper::{Html, Selector};
use std::error::Error;

#[derive(Clone)] // Derive the Clone trait for Comment
pub struct Comment {
    pub hnuser: String,
    pub title: String,
    pub commtext: String,
}

pub fn hnscraper(username: &str) -> Result<Vec<Comment>, Box<dyn Error>> {
    // Define the URL to fetch with the specified username
    let url = format!("https://news.ycombinator.com/threads?id={}", username);

    // Send an HTTP GET request and retrieve the response
    let response = reqwest::blocking::get(url)?;

    // Check if the request was successful
    if !response.status().is_success() {
        eprintln!("HTTP request failed with status code: {:?}", response.status());
        return Ok(Vec::new());
    }
    // Read the response body as a string
    let html = response.text()?;

    // Parse the HTML using the parse_html function
    parse_html(&html)
}

pub fn parse_html(html: &str) -> Result<Vec<Comment>, Box<dyn Error>> {
    // Parse the HTML content
    let document = Html::parse_document(html);

    // Define selectors for the elements you want to extract
    let title_selector = Selector::parse("span.age[title]").unwrap();
    let commtext_selector = Selector::parse("div.comment span.commtext.c00:not(.reply)").unwrap();
    let hnuser_selector = Selector::parse("a.hnuser").unwrap();

    // Extract and print the desired elements
    let mut comments = Vec::new();

    for element in document.select(&title_selector).zip(document.select(&commtext_selector).zip(document.select(&hnuser_selector))) {
        let (title, (commtext, hnuser)) = element;
        let hnuser_text = hnuser.text().collect::<String>();

        // Convert title_text to String
        let title_text = title.value().attr("title").unwrap_or_default().to_string();

        let commtext_text = remove_reply_suffix(&commtext.text().collect::<String>());

        // Create a new Comment struct
        let comment = Comment {
            hnuser: hnuser_text,
            title: title_text,
            commtext: commtext_text,
        };

        // Add the comment to the comments vector
        comments.push(comment);
    }

    Ok(comments)
}

fn remove_reply_suffix(text: &str) -> String {
    // Remove any whitespace at the end of the text
    let trimmed_text = text.trim();

    // Check if the text ends with "reply"
    if trimmed_text.ends_with("reply") {
        // Remove "reply" and any trailing whitespace
        trimmed_text[..trimmed_text.len() - "reply".len()].trim().to_string()
    } else {
        // If it doesn't end with "reply," return the original text
        trimmed_text.to_string()
    }
}
