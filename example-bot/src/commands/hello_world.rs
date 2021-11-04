use crate::config::Config;
use crate::errors::Error;
use matrix_sdk::ruma::events::{room::message::MessageEventContent, AnyMessageEventContent};
use matrix_sdk::ruma::RoomId;
use matrix_sdk::Client;
use mrsbfh::commands::command;
use std::sync::Arc;
use tokio::sync::Mutex;

#[command(help = "`!hello_world` - Prints \"hello world\".")]
pub async fn hello_world<'a>(
    _client: Client,
    tx: mrsbfh::Sender,
    _config: Arc<Mutex<Config<'a>>>,
    _sender: String,
    _room_id: RoomId,
    mut _args: Vec<&str>,
) -> Result<(), Error>
where
    Config<'a>: mrsbfh::config::Loader + Clone,
{
    let content =
        AnyMessageEventContent::RoomMessage(MessageEventContent::notice_plain("Hello World!"));

    tx.send(content).await?;
    Ok(())
}
