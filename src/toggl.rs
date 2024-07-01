use reqwest::{Client, Method};
use reqwest::header::{CONTENT_TYPE, AUTHORIZATION};
use serde::Deserialize;
use std::fs;
use base64::{engine::general_purpose, Engine as _};
use chrono::{Utc, Duration};

#[derive(Deserialize, Debug)]
struct TimeEntry {
    at: String,
    description: Option<String>,
    duration: i64,
    id: u64,
    start: String, 
    stop: Option<String>,
}

#[derive(Deserialize)]
struct Config {
    api: ApiConfig,
}

#[derive(Deserialize)]
struct ApiConfig {
    token: String,
}

pub async fn get_time_entries() -> Result<(), reqwest::Error> {
    // Read the config file
    let config_content = fs::read_to_string("config.toml")
        .expect("Failed to read config file");
    let config: Config = toml::from_str(&config_content)
        .expect("Failed to parse config file");

    // Extract the API token
    let api_token = config.api.token;
    let authorization_value = format!("Basic {}", general_purpose::STANDARD.encode(format!("{}:api_token", api_token)));

    // Calculate the Unix timestamp for one month ago
    let one_month_ago = Utc::now() - Duration::days(90);
    let since_timestamp = one_month_ago.timestamp();

    // Create the HTTP client and make the request
    let client = Client::new();
    let url = format!("https://api.track.toggl.com/api/v9/me/time_entries?since={}", since_timestamp);
    let response = client.request(Method::GET, url)
        .header(CONTENT_TYPE, "application/json")
        .header(AUTHORIZATION, authorization_value)
        .send()
        .await?;

    // Handle the JSON response as an array
    let time_entries: Vec<TimeEntry> = response.json().await?;
    
    println!("{:#?}", time_entries);

    Ok(())
}
