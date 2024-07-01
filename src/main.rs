use log::info;

mod token;
mod youtrack;

#[tokio::main]
async fn main() {
    simple_logger::init().unwrap();

    let user = youtrack::get_current_user().await.unwrap();
    info!("User: {:#?}", user);

    youtrack::get_workitems("SO-106").await;

    //youtrack::send_post().await;
}
