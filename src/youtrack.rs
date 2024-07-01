use chrono::{serde::ts_milliseconds, DateTime, NaiveDate, Utc};
use std::collections::HashMap;

use crate::token::AUTH_TOKEN;

use log::info;
use serde::{Deserialize, Serialize};

const BASE_URL: &str = "https://systemscape.youtrack.cloud";

#[derive(Serialize, Deserialize, Debug)]
pub struct IssueWorkItem {
    #[serde(skip_serializing)]
    id: String,
    author: User,
    creator: User,
    text: String,
    #[serde(rename = "type")]
    item_type: WorkItemType,
    #[serde(with = "ts_milliseconds")]
    created: DateTime<Utc>,
    duration: Duration,
    #[serde(with = "ts_milliseconds")]
    date: DateTime<Utc>,
}
#[derive(Serialize, Deserialize, Debug)]
pub struct Duration {
    minutes: u32,
}
#[derive(Serialize, Deserialize, Debug)]
pub struct IssueId {
    #[serde(rename = "idReadable")]
    id_readable: String,
}
#[derive(Serialize, Deserialize, Debug)]
pub struct WorkItemType {
    id: String,
    name: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct User {
    pub login: String,
    pub id: String,
}

pub async fn create_work_item(item: IssueWorkItem) {
    let client = reqwest::Client::new();
    let issue_id = "SO-106";

    let res = client
    .post(format!("{BASE_URL}/api/issues/{issue_id}/timeTracking/workItems?fields=author(id,login),creator(id,login),date,created(minutes),duration(minutes),id,name,text,type(id,name),issue(idReadable)"))
    .bearer_auth(AUTH_TOKEN)
    .header("Content-Type", "application/json")
    .body(serde_json::to_string(&item).unwrap())
    .send()
    .await;

    info!(
        "Got result: {:#?}",
        res.unwrap().json::<serde_json::Value>().await
    );
}

pub async fn get_workitems(issue_id: &str) {
    let url = format!("{BASE_URL}/api/issues/{issue_id}/timeTracking/workItems?fields=author(id,login),creator(id,login),date,created(minutes),duration(minutes),id,name,text,type(id,name),issue(idReadable)");

    let res = perform_request(&url).await.unwrap().text().await.unwrap();
    info!("Got res: {:#?}", res);

    let item: Vec<IssueWorkItem> = serde_json::from_str(&res).unwrap();
    info!("Got WorkItemType: {:#?}", item);
}

pub async fn perform_request(url: &str) -> Result<reqwest::Response, reqwest::Error> {
    let client = reqwest::Client::new();
    client.get(url).bearer_auth(AUTH_TOKEN).send().await
}

pub async fn get_current_user() -> Result<User, String> {
    let client = reqwest::Client::new();

    let res = client
        .get(format!("{BASE_URL}/api/users/me?fields=id,login"))
        .bearer_auth(AUTH_TOKEN)
        .send()
        .await;

    let res = res.unwrap().text().await.unwrap();
    info!("Got res: {:#?}", res);

    let user: User = serde_json::from_str(&res).unwrap();
    Ok(user)
}
