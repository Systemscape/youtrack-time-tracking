use chrono::{serde::ts_milliseconds, DateTime, Utc};

use crate::token::AUTH_TOKEN;

use log::info;
use serde::{Deserialize, Serialize};

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

pub async fn create_work_item(issue_id: &str, item: IssueWorkItem) {
    let client = reqwest::Client::new();

    let res = client
        .post(format!(
            "{BASE_URL}/api/issues/{issue_id}/timeTracking/workItems?fields={WORK_ITEMS_FIELDS}"
        ))
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

pub async fn get_workitems(issue_id: String) -> Result<Vec<IssueWorkItem>, reqwest::Error> {
    info!("get_workitems for issue_id {}", &issue_id);
    let url = format!(
        "{BASE_URL}/api/issues/{issue_id}/timeTracking/workItems?fields={WORK_ITEMS_FIELDS}"
    );

    let res = perform_request(&url).await.unwrap();
    let res = res.error_for_status()?.text().await.unwrap();
    let items: Vec<IssueWorkItem> = serde_json::from_str(&res).unwrap();

    Ok(items)
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

#[cfg(test)]
mod test {
    use log::info;

    use crate::youtrack;

    #[tokio::test]
    async fn test_serde() {
        simple_logger::init().unwrap();

        let user = youtrack::get_current_user().await.unwrap();
        info!("User: {:#?}", user);

        youtrack::get_workitems("SO-106").await;
    }
}
