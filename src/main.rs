use log::info;

mod toggl;

#[tokio::main]
async fn main() -> Result<(), reqwest::Error> {
    info!("Getting time entries...");

    let time_entries: Vec<toggl::TimeEntry> = toggl::get_time_entries(90).await?;
    println!("{:#?}", time_entries);

    Ok(())
}
