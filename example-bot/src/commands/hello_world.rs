use matrix_sdk::events::{room::message::MessageEventContent, AnyMessageEventContent};
use mrsbfh::commands::command;
use mrsbfh::config::Config;
use std::error::Error;

#[command(help = "`!hello_world` - Prints \"hello world\".")]
pub async fn hello_world<C: Config>(
    tx: mrsbfh::Sender,
    _config: C,
    _sender: String,
    mut _args: Vec<&str>,
) -> Result<(), Box<dyn Error>> {
    let content =
        AnyMessageEventContent::RoomMessage(MessageEventContent::notice_plain("Hello World!"));

    tx.send(content).await?;
    Ok(())
}
