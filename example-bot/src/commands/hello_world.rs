use crate::errors::Error;
use matrix_sdk::ruma::events::room::message::RoomMessageEventContent;
use mrsbfh::commands::command;
use mrsbfh::commands::extract::Extension;

#[command(help = "`!hello_world` - Prints \"hello world\".")]
pub async fn hello_world<'a>(Extension(tx): Extension<mrsbfh::Sender>) -> Result<(), Error> {
    let content = RoomMessageEventContent::notice_plain("Hello World!");

    tx.lock().await.send(content).await?;
    Ok(())
}
