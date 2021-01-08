use crate::config::Config;
use crate::errors::Error;
use matrix_sdk::events::{room::message::MessageEventContent, AnyMessageEventContent};
use mrsbfh::commands::command;

#[command(help = "`!hello_world` - Prints \"hello world\".")]
pub async fn hello_world<'a>(
    mut tx: mrsbfh::Sender,
    _config: Config<'a>,
    _sender: String,
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
