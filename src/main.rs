use dialoguer::{theme::ColorfulTheme, Confirm};
use futures::{stream, StreamExt};
use log::{debug, info};
use serde::Deserialize;
use std::{
    collections::{HashMap, HashSet},
    fs,
    process::exit,
};
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

    // Get the current youtrack user for later use
    let user = youtrack::get_current_user().await.unwrap();
    info!("User: {:#?}", user);

    // Get all toggl time entries of the last X days.
    info!("Getting toggl time entries");
    let time_entries: Vec<toggl::TimeEntry> = toggl::get_time_entries(90, config.toggl_api).await?;

    // Create a regex to extract the Issue ID from the time entry
    let re = Regex::new(REGEX_STRING).unwrap();

    // Filter time entries that match the regex and return iterator of ExtendedTimeEntry with that data
    let time_entries = time_entries.into_iter().filter_map(|entry| {
        re.captures(&entry.description.clone().unwrap_or("".to_string()))
            .and_then(|x| {
                // Issue ID is in the first capture
                let issue_id = x.get(1)?.as_str().to_string();
                // Description text is everything that follows, i.e., the second capture
                let description = x.get(2)?.as_str().to_string();
                Some(ExtendedTimeEntry {
                    toggl_time_entry: entry,
                    issue_id,
                    description,
                })
            })
    });

    // Create a HashMap that matches each unique Issue ID with a Vec of toggl time entries
    let mut issue_id_time_entries_map: HashMap<String, Vec<ExtendedTimeEntry>> = HashMap::new();
    for entry in time_entries {
        debug!("ID: {}, description: {}", entry.issue_id, entry.description);
        issue_id_time_entries_map
            .entry(entry.issue_id.clone())
            .or_default()
            .push(entry);
    }

    // The unique Issue IDs from all toggl time entries correspond to the HashMap's keys.
    let unique_issue_ids = issue_id_time_entries_map
        .keys()
        .cloned()
        .collect::<HashSet<_>>();

    debug!("unique_issue_ids: {:#?}", &unique_issue_ids);

    // For all unique Issue IDs, obtain a Vec of associated IssueWorkItems.
    // The association is represented by a HashMap. This should happen asynchronously (slow web reqs).
    let work_items_map = stream::iter(unique_issue_ids)
        .filter_map(|issue_id| {
            async move {
                match youtrack::get_workitems(issue_id.clone()).await {
                    // If WorkItems are obtained for this issue ID, return them.
                    Ok(work_items) => Some((issue_id.clone(), work_items)),
                    // Otherwise do not include that Issue ID in the HashMap.
                    Err(e) => {
                        log::error!("Could not obtain WorkItem for issue_id {}: {}", issue_id, e);
                        None
                    }
                }
            }
        })
        .collect::<HashMap<String, Vec<_>>>()
        .await;

    // The Issue IDs that exist on youtrack correspond to the keys of the HashMap.
    let existent_issue_ids = work_items_map.keys().cloned().collect::<Vec<_>>();

    log::debug!("Existent issue_ids: {:#?}", &existent_issue_ids);

    log::debug!("Got work_items without errors: {:#?}", &work_items_map);

    // Filter out all toggl time entries that have an Issue ID associated that is not valid on youtrack
    let existent_issue_id_time_entries_map = issue_id_time_entries_map
        .into_iter()
        .filter(|(key, _)| existent_issue_ids.contains(key))
        .collect::<HashMap<String, Vec<_>>>();

    // Obtain the missing (i.e., not present on youtrack) toggl time entries.
    // This is done by checking whether the toggl time entry's ID is present
    // in the "text" field of any youtrack work item.
    let missing_time_entries = existent_issue_id_time_entries_map
        .into_iter()
        .filter_map(|(issue_id, time_entries_for_id)| {
            match work_items_map.get(&issue_id) {
                // If not a single work item exists for this Issue ID,
                // all toggl entries for that Issue ID need to be created.
                None => Some(time_entries_for_id),
                // If work items exist for this Issue ID,
                // filter out any toggl time entries that are already present.
                // This is the case when the toggl entry ID is contained in work item "text" field.
                Some(work_items) => {
                    let missing_time_entries_for_id = time_entries_for_id
                        .into_iter()
                        .filter(|time_entry_for_id|
                            // Keep only if not any work_item contains the ID in the "text" field 
                            !work_items.iter().any(|item| {
                                item.text
                                    .contains(&time_entry_for_id.toggl_time_entry.id.to_string())
                            }))
                        .collect::<Vec<_>>();

                    if missing_time_entries_for_id.is_empty() {
                        None
                    } else {
                        Some(missing_time_entries_for_id)
                    }
                }
            }
        })
        .collect::<Vec<_>>();

    log::info!("missing_time_entries: {:#?}", missing_time_entries);

    if !Confirm::with_theme(&ColorfulTheme::default())
        .with_prompt("Do you want to continue?")
        .interact()
        .unwrap()
    {
        exit(1);
    }

    // Create a work item for all missing time entries
    stream::iter(missing_time_entries)
        .for_each(|entries| async {
            for entry in entries {
                let duration_minutes = entry.toggl_time_entry.duration as u32 / 60;

                // Youtrack cannot have durations below 1 minute but toggl tracks with second accuracy
                if duration_minutes > 0 {
                    let work_item = IssueWorkItem {
                        id: "".to_string(),
                        author: user.clone(),
                        creator: user.clone(),
                        text: entry.toggl_time_entry.id.to_string() + " - " + &entry.description,
                        created: chrono::Local::now().into(),
                        duration: Duration {
                            minutes: duration_minutes,
                        },
                        date: entry.toggl_time_entry.start.into(),
                        issue: None,
                    };

                    debug!(
                        "Issue {} - creating work_item: {:#?}",
                        &entry.issue_id, &work_item
                    );
                    youtrack::create_work_item(&entry.issue_id, work_item).await
                } else {
                    log::warn!("Duration not > 0. Skipping entry: {:#?}", entry);
                }
            }
        })
        .await;

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
    youtrack::get_workitems("ABC-123".to_string()).await;
}
