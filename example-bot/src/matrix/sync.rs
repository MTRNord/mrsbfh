use crate::commands::match_command;
use crate::config::Config;
use mrsbfh::matrix_sdk::{
    events::{
        room::member::MemberEventContent, room::message::MessageEventContent, StrippedStateEvent,
        SyncMessageEvent,
    },
    Client, EventEmitter, SyncRoom,
};
use mrsbfh::matrix_sdk_common_macros::async_trait;
use tokio::sync::mpsc;
use tracing::*;

#[derive(Debug, Clone)]
pub struct Bot {
    client: Client,
    config: Config<'static>,
}

impl Bot {
    pub fn new(client: Client, config: Config<'static>) -> Self {
        Self {
            client,
            config: config.clone(),
        }
    }
}

#[mrsbfh::commands::commands]
#[mrsbfh::utils::autojoin]
#[async_trait]
impl EventEmitter for Bot {
    async fn on_room_message(&self, room: SyncRoom, event: &SyncMessageEvent<MessageEventContent>) {
        println!("message example")
    }

    async fn on_stripped_state_member(
        &self,
        room: SyncRoom,
        room_member: &StrippedStateEvent<MemberEventContent>,
        _: Option<MemberEventContent>,
    ) {
        println!("autojoin example")
    }
}
