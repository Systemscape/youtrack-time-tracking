use chrono::{serde::ts_milliseconds, DateTime, Utc};
use log::debug;
use serde::{Deserialize, Serialize};

use crate::ApiConfig;

const BASE_URL: &str = "https://systemscape.youtrack.cloud";
const WORK_ITEMS_FIELDS: &str = "author(id,login),creator(id,login),date,created(minutes),duration(minutes),id,name,text,issue(idReadable)";
//const WORK_ITEMS_FIELDS: &str = "author(id,login),creator(id,login),date,created(minutes),duration(minutes),id,name,text,type(id,name),issue(idReadable)";

#[derive(Serialize, Deserialize, Debug)]
pub struct IssueWorkItem {
    #[serde(skip_serializing)]
    pub id: String,
    pub author: User,
    pub creator: User,
    pub text: String,
    //#[serde(rename = "type")]
    //pub item_type: WorkItemType,
    #[serde(with = "ts_milliseconds")]
    pub created: DateTime<Utc>,
    pub duration: Duration,
    #[serde(with = "ts_milliseconds")]
    pub date: DateTime<Utc>,
    #[serde(skip_serializing)]
    pub issue: Option<IssueId>, // Read-only field, store as option
}
#[derive(Serialize, Deserialize, Debug)]
pub struct Duration {
    pub minutes: u32,
}
#[derive(Serialize, Deserialize, Debug)]
pub struct IssueId {
    #[serde(rename = "idReadable")]
    pub id_readable: String,
}
#[derive(Serialize, Deserialize, Debug)]
pub struct WorkItemType {
    pub id: String,
    pub name: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct User {
    pub login: String,
    pub id: String,
}

pub async fn create_work_item(issue_id: &str, item: IssueWorkItem, config: ApiConfig) {
    let client = reqwest::Client::new();

    let res = client
        .post(format!(
            "{BASE_URL}/api/issues/{issue_id}/timeTracking/workItems?fields={WORK_ITEMS_FIELDS}"
        ))
        .bearer_auth(config.token)
        .header("Content-Type", "application/json")
        .body(serde_json::to_string(&item).unwrap())
        .send()
        .await;

    debug!(
        "create_work_item - got result: {:#?}",
        res.unwrap().json::<serde_json::Value>().await
    );
}

pub async fn get_workitems(issue_id: String, config: ApiConfig) -> Result<Vec<IssueWorkItem>, reqwest::Error> {
    debug!("get_workitems for issue_id {}", &issue_id);
    let url = format!(
        "{BASE_URL}/api/issues/{issue_id}/timeTracking/workItems?fields={WORK_ITEMS_FIELDS}"
    );

    let res = perform_request(&url, config).await.unwrap();
    let res = res.error_for_status()?.text().await.unwrap();
    let items: Vec<IssueWorkItem> = serde_json::from_str(&res).unwrap();

    Ok(items)
}

pub async fn perform_request(url: &str, config: ApiConfig) -> Result<reqwest::Response, reqwest::Error> {
    let client = reqwest::Client::new();
    client.get(url).bearer_auth(config.token).send().await
}

pub async fn get_current_user(config: ApiConfig) -> Result<User, String> {
    let client = reqwest::Client::new();

    let res = client
        .get(format!("{BASE_URL}/api/users/me?fields=id,login"))
        .bearer_auth(config.token)
        .send()
        .await;

    let res = res.unwrap().text().await.unwrap();
    debug!("get_current_user - got res: {:#?}", res);

    let user: User = serde_json::from_str(&res).unwrap();
    Ok(user)
}

#[cfg(test)]
mod test {
    use log::info;
    use std::fs;
    
    use crate::Config;
    use crate::youtrack;

    #[tokio::test]
    async fn test_serde() {
        simple_logger::init().unwrap();

        // Read the config file
        let config_content = fs::read_to_string("config.toml").expect("Failed to read config file");
        let config: Config = toml::from_str(&config_content).expect("Failed to parse config file");


        let user = youtrack::get_current_user(config.youtrack_api.clone()).await.unwrap();
        info!("User: {:#?}", user);

        youtrack::get_workitems("SO-106".to_string(), config.youtrack_api.clone()).await;
    }
}
