use base64::{engine::general_purpose, Engine as _};
use chrono::{Duration, Utc};
use reqwest::header::{AUTHORIZATION, CONTENT_TYPE};
use reqwest::{Client, Method};
use serde::Deserialize;

use crate::ApiConfig;

#[derive(Deserialize, Debug)]
pub struct TimeEntry {
    pub at: String,
    pub description: Option<String>,
    pub duration: i64,
    pub id: u64,
    pub start: String,
    pub stop: Option<String>,
}

pub async fn get_time_entries(
    days: i64,
    config: ApiConfig,
) -> Result<Vec<TimeEntry>, reqwest::Error> {
    // Extract the API token
    let api_token = config.token;
    let authorization_value = format!(
        "Basic {}",
        general_purpose::STANDARD.encode(format!("{}:api_token", api_token))
    );

    // Calculate the Unix timestamp for one month ago
    let one_month_ago = Utc::now() - Duration::days(days);
    let since_timestamp = one_month_ago.timestamp();

    // Create the HTTP client and make the request
    let client = Client::new();
    let url = format!(
        "https://api.track.toggl.com/api/v9/me/time_entries?since={}",
        since_timestamp
    );
    let response = client
        .request(Method::GET, url)
        .header(CONTENT_TYPE, "application/json")
        .header(AUTHORIZATION, authorization_value)
        .send()
        .await?;

    // Handle the JSON response as an array
    let time_entries: Vec<TimeEntry> = response.json().await?;

    Ok(time_entries)
}
