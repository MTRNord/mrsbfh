use crate::commands::match_command;
use crate::Config;
use matrix_sdk::room::Room;
use matrix_sdk::ruma::events::room::message::OriginalSyncRoomMessageEvent;
use matrix_sdk::Client;
use std::sync::Arc;
use tokio::sync::Mutex;

#[mrsbfh::commands::commands]
pub(crate) async fn on_room_message(
    event: OriginalSyncRoomMessageEvent,
    room: Room,
    client: Client,
    config: Arc<Mutex<Config<'static>>>,
) {
}
