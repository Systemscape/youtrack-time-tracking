use log::info;

mod token;
mod youtrack;

mod toggl;

#[tokio::main]
async fn main() -> Result<(), reqwest::Error> {
    info!("Getting time entries...");

    let time_entries: Vec<toggl::TimeEntry> = toggl::get_time_entries(90).await?;
    println!("{:#?}", time_entries);

    simple_logger::init().unwrap();

    let user = youtrack::get_current_user().await.unwrap();
    info!("User: {:#?}", user);

    youtrack::get_workitems("SO-106").await;

    //youtrack::send_post().await;

    Ok(())
}
