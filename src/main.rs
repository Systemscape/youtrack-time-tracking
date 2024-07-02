use futures::{stream, StreamExt, TryFutureExt};
use log::info;
use serde::Deserialize;
use std::{collections::HashMap, error, fs};
use youtrack::{Duration, IssueWorkItem};

use regex::Regex;

mod token;
mod youtrack;

mod toggl;

const REGEX_STRING: &str = r"(\w+-\d+) (.*)";

#[derive(Deserialize)]
struct Config {
    toggl_api: ApiConfig,
    youtrack_api: ApiConfig,
}

#[derive(Deserialize)]
struct ApiConfig {
    token: String,
}

#[derive(Debug)]
struct ExtendedTimeEntry {
    toggl_time_entry: toggl::TimeEntry,
    issue_id: String,
    description: String,
}

#[tokio::main]
async fn main() -> Result<(), reqwest::Error> {
    simple_logger::init().unwrap();

    // Read the config file
    let config_content = fs::read_to_string("config.toml").expect("Failed to read config file");
    let config: Config = toml::from_str(&config_content).expect("Failed to parse config file");

    info!("Getting time entries");

    let time_entries: Vec<toggl::TimeEntry> = toggl::get_time_entries(90, config.toggl_api).await?;
    //println!("{:#?}", time_entries);

    info!("Filtering by Regex");

    let re = Regex::new(REGEX_STRING).unwrap();

    // Filter time entries that match the regex and return iterator of ExtendedTimeEntry with that data
    let mut time_entries = time_entries.into_iter().filter_map(|entry| {
        re.captures(&entry.description.clone().unwrap_or("".to_string()))
            .and_then(|x| {
                let issue_id = x.get(1)?.as_str().to_string();
                let description = x.get(2)?.as_str().to_string();
                Some(ExtendedTimeEntry {
                    toggl_time_entry: entry,
                    issue_id,
                    description,
                })
            })
    });

    let mut unique_issue_ids: HashMap<String, Vec<ExtendedTimeEntry>> = HashMap::new();
    for entry in time_entries {
        info!("ID: {}, description: {}", entry.issue_id, entry.description);
        unique_issue_ids
            .entry(entry.issue_id.clone())
            .or_default()
            .push(entry);
    }

    info!("Hashmap: {:#?}", unique_issue_ids);

    //return Ok(());

    let user = youtrack::get_current_user().await.unwrap();
    info!("User: {:#?}", user);

    let work_items = stream::iter(unique_issue_ids)
        .filter_map(|(issue_id, _)| async move {
            match youtrack::get_workitems(issue_id.clone()).await {
                Ok(work_items) => Some(stream::iter(work_items)), // necessary to use flatten later
                Err(e) => {
                    log::error!("Could not obtain WorkItem for issue_id {}: {}", issue_id, e);
                    None
                }
            }
        })
        .flatten()
        .collect::<Vec<_>>()
        .await;

    let work_items_map = HashMap::new();
    work_items.into_iter().for_each(|item| {
        work_items_map
            .entry(item.issue.id_readable)
            .or_default()
            .push(item)
    });

    log::error!("Got work_items without errors: {:#?}", &work_items);

    let time_entries = unique_issue_ids
        .into_iter()
        .filter_map(|(issue_id, time_entry)| {
            match work_items.get(&issue_id) {
                None => Some(time_entry),
                Some(work_item) => {}
            }
            // Make sure that not any of the work_items already contains that id
            !work_items.iter().any(|work_item| {
                work_item
                    .text
                    .contains(&time_entry.toggl_time_entry.id.to_string())
            })
        })
        .collect::<Vec<_>>();

    log::error!("time_entries left: {:#?}", time_entries);
    return Ok(());

    /*
    Some(entries.into_iter().filter(move |entry| {
                !work_items
                    .iter()
                    .any(|item| item.text.contains(&entry.toggl_time_entry.id.to_string()))
            })))

            //log::error!("Could not obtain WorkItem for issue_id {}: {}",&issue_id,e);

    work_items
        .for_each(|entries| async {
            for entry in entries {
                let work_item = IssueWorkItem {
                    id: "".to_string(),
                    author: user.clone(),
                    creator: user.clone(),
                    text: entry.toggl_time_entry.id.to_string(),
                    created: chrono::Local::now().into(),
                    duration: Duration {
                        minutes: entry.toggl_time_entry.duration as u32 / 60,
                    },
                    date: entry.toggl_time_entry.start.into(),
                };

                info!(
                    "Issue {} - creating work_item: {:#?}",
                    &entry.issue_id, &work_item
                );
                youtrack::create_work_item(&entry.issue_id, work_item).await
            }
        })
        .await;

    */

    //youtrack::send_post().await;

    Ok(())
}

#[cfg(test)]
mod test {
    use regex::Regex;

    use crate::REGEX_STRING;

    #[tokio::test]
    async fn test_regex() {
        let re = Regex::new(REGEX_STRING).unwrap();
        let caps = re.captures("DIT-2 My Description").unwrap();

        assert_eq!(caps.get(1).unwrap().as_str(), "DIT-2");
        assert_eq!(caps.get(2).unwrap().as_str(), "My Description");
    }
}

#[tokio::test]
async fn test_wrong_issue_id() {
    youtrack::get_workitems("ABC-123").await;
}
