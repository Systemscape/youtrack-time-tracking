use chrono::{serde::ts_milliseconds, DateTime, NaiveDate, Utc};
use std::collections::HashMap;

use crate::token::AUTH_TOKEN;

use log::info;
use serde::{Deserialize, Serialize};

const BASE_URL: &str = "https://systemscape.youtrack.cloud";

#[derive(Serialize, Deserialize, Debug)]
pub struct IssueWorkItem {
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
    #[serde(rename = "issue")]
    issue_id: IssueId,
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

pub async fn send_post() {
    let mut map = HashMap::new();
    map.insert("lang", "rust");
    map.insert("body", "json");

    //
    //https://systemscape.youtrack.cloud/api/issues/2-35/timeTracking/workItems?fields=author(id,name),creator(id,name),date,duration(id,minutes,presentation),id,name,text,type(id,name)
    let body = r###"
{
  "usesMarkdown": true,
  "text": "I keep on testing *samples*.",
  "date": 1539000000000,
  "author": {
    "id": "24-0"
  },
  "duration": {
    "minutes": 120
  },
  "type": {
    "id": "49-0"
  }
}"###;

    let client = reqwest::Client::new();
    let issue_id = "SO-106";

    let res = client
        .get(format!("{BASE_URL}/api/users/me?fields=id"))
        .bearer_auth(AUTH_TOKEN)
        .send()
        .await;

    let res = res.unwrap().json::<serde_json::Value>().await.unwrap();
    info!("Got res: {:#?}", res);

    let user_id = res.get("id").unwrap().as_str().unwrap();

    info!("user_id is {user_id}");

    let res = client
    .get(format!("{BASE_URL}/api/issues/{issue_id}/timeTracking/workItems?fields=author(id,name),creator(id,name),date,duration(id,minutes,presentation),id,name,text,type(id,name)"))
    .bearer_auth(AUTH_TOKEN)
    .send()
    .await;

    info!(
        "Got result: {:#?}",
        res.unwrap().json::<serde_json::Value>().await
    );

    let res = client
    .post(format!("{BASE_URL}/api/issues/{issue_id}/timeTracking/workItems?fields=author(name),creator(name),date,duration(id,minutes,presentation),id,name,text,type(id,name)"))
    .bearer_auth(AUTH_TOKEN)
    .header("Content-Type", "application/json")
    .body(r##"{
  "usesMarkdown": true,
  "text": "I keep on testing *samples*.",
  "date": 1539000000000,
  "author": {
    "name": "jdickert",
    "id": "1-8"
  },
  "duration": {
    "minutes": 120
  },
  "type": {
    "id": "139-0",
    "name": "Development"
  }
}"##)
    .send()
    .await;

    info!(
        "Got result: {:#?}",
        res.unwrap().json::<serde_json::Value>().await
    );

    /*

    let res = client
        .post("http://httpbin.org/post")
        .json(&map)
        .send()
        .await?;*/
}

pub async fn get_workitems(issue_id: &str) {
    let url = format!("{BASE_URL}/api/issues/{issue_id}/timeTracking/workItems?fields=author(id,login),creator(id,login),date,created(minutes),duration(minutes),id,name,text,type(id,name),issue(idReadable)");

    let res = perform_request(&url).await.unwrap();
    info!("Got result: {:#?}", res.json::<serde_json::Value>().await);

    let res = perform_request(&url).await.unwrap().text().await.unwrap();
    info!("Got res: {:#?}", res);
    let item: Vec<IssueWorkItem> = serde_json::from_str(&res).unwrap();
    info!("Got WorkItemType: {:#?}", item);
}

pub async fn perform_request(url: &str) -> Result<reqwest::Response, reqwest::Error> {
    let client = reqwest::Client::new();
    client.get(url).bearer_auth(AUTH_TOKEN).send().await
}

#[derive(Serialize, Deserialize, Debug)]
pub struct User {
    pub login: String,
    pub id: String,
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
