use futures::{stream, StreamExt};
use log::info;
use serde::Deserialize;
use serde_json::error;
use std::fs;

use regex::Regex;

mod token;
mod youtrack;

mod toggl;

#[derive(Deserialize)]
struct Config {
    toggl_api: ApiConfig,
    youtrack_api: ApiConfig,
}

#[derive(Deserialize)]
struct ApiConfig {
    token: String,
}

#[tokio::main]
async fn main() -> Result<(), reqwest::Error> {
    simple_logger::init().unwrap();

    // Read the config file
    let config_content = fs::read_to_string("config.toml").expect("Failed to read config file");
    let config: Config = toml::from_str(&config_content).expect("Failed to parse config file");

    info!("Getting time entries...");

    let time_entries: Vec<toggl::TimeEntry> = toggl::get_time_entries(90, config.toggl_api).await?;
    println!("{:#?}", time_entries);

    let re = Regex::new(r"(\w+-\d+) (.*)").unwrap();
    let caps = re
        .captures("DIT-2 Requirements und Zeitplan zusammenführen")
        .unwrap();
    log::error!("captures: {:#?}", caps);

    let issue_id = caps.get(1).unwrap().as_str();
    let text = caps.get(2).unwrap().as_str();

    let time_entries = time_entries.iter().filter(|entry| {
        entry
            .description.clone()
            .map(|text| re.captures(text.as_ref()).unwrap().get(1).is_some())
            .is_some()
    });

    

    let user = youtrack::get_current_user().await.unwrap();
    info!("User: {:#?}", user);

    let issue_ids: Vec<String> = vec!["SO-106".to_string(), "SO-100".to_string()];

    let work_items = stream::iter(issue_ids)
        .map(|issue_id| async move { youtrack::get_workitems(&issue_id).await })
        .buffer_unordered(10);

    work_items
        .for_each(|b| async move { info!("got items: {:#?}", b) })
        .await;

    //youtrack::send_post().await;

    Ok(())
}
