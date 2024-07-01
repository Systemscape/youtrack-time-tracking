use log::info;

mod toggl;

#[tokio::main]
async fn main() -> Result<(), reqwest::Error> {
    info!("Getting time entries...");

    toggl::get_time_entries().await?;

    Ok(())
}
