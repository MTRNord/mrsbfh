use crate::config::Config;
use crate::errors::Error;
use matrix_sdk::ruma::events::{room::message::MessageEventContent, AnyMessageEventContent};
use mrsbfh::commands::command;
use mrsbfh::commands::extract::Extension;
use std::sync::Arc;

#[command(help = "`!hello_world` - Prints \"hello world\".")]
pub async fn hello_world<'a>(Extension(tx): Extension<Arc<mrsbfh::Sender>>) -> Result<(), Error>
where
    Config<'a>: mrsbfh::config::Loader + Clone,
{
    let content =
        AnyMessageEventContent::RoomMessage(MessageEventContent::notice_plain("Hello World!"));

    tx.send(content).await?;
    Ok(())
}
