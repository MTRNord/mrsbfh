use crate::commands::match_command;
use crate::Config;
use matrix_sdk::Client;
use matrix_sdk::{
    room::Room,
    ruma::events::{room::message::MessageEventContent, SyncMessageEvent},
};
use std::sync::Arc;
use tokio::sync::Mutex;

#[mrsbfh::commands::commands]
pub(crate) async fn on_room_message(
    event: SyncMessageEvent<MessageEventContent>,
    room: Room,
    client: Client,
    config: Arc<Mutex<Config<'static>>>,
) {
    println!("message example")
}
