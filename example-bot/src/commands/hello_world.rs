use crate::errors::Error;
use matrix_sdk::ruma::events::{room::message::MessageEventContent, AnyMessageEventContent};
use mrsbfh::commands::command;
use mrsbfh::commands::extract::Extension;
use mrsbfh::tokio::sync::Mutex;
use std::sync::Arc;

#[command(help = "`!hello_world` - Prints \"hello world\".")]
pub async fn hello_world<'a>(
    Extension(tx): Extension<Arc<Mutex<mrsbfh::Sender>>>,
) -> Result<(), Error> {
    let content =
        AnyMessageEventContent::RoomMessage(MessageEventContent::notice_plain("Hello World!"));

    tx.lock().await.send(content).await?;
    Ok(())
}
