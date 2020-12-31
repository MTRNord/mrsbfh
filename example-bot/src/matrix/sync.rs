use crate::commands::match_command;
use crate::config::Config;
use mrsbfh::matrix_sdk::{
    events::{
        room::message::{MessageEventContent, TextMessageEventContent},
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

#[async_trait]
impl EventEmitter for Bot {
    async fn on_room_message(&self, room: SyncRoom, event: &SyncMessageEvent<MessageEventContent>) {
        if let SyncRoom::Joined(room) = room {
            let msg_body = if let SyncMessageEvent {
                content: MessageEventContent::Text(TextMessageEventContent { body: msg_body, .. }),
                ..
            } = event
            {
                msg_body.clone()
            } else {
                String::new()
            };
            if msg_body.is_empty() {
                return;
            }

            let sender = event.sender.clone().to_string();

            let (tx, mut rx) = mpsc::channel(100);
            let room_id = room.read().await.clone().room_id;

            let cloned_config = self.config.clone();
            tokio::spawn(async move {
                let mut split = msg_body.split_whitespace();

                let command_raw = split.next().expect("This is not a command");
                let command = command_raw.to_lowercase();
                info!("Got command: {}", command);

                // Make sure this is immutable
                let args: Vec<&str> = split.collect();
                if let Err(e) = match_command(
                    command.replace("!", "").as_str(),
                    cloned_config.clone(),
                    tx,
                    sender,
                    args,
                )
                .await
                {
                    error!("{}", e);
                }
            });

            while let Some(v) = rx.recv().await {
                if let Err(e) = self
                    .client
                    .clone()
                    .room_send(&room_id.clone(), v, None)
                    .await
                {
                    error!("{}", e);
                }
            }
        }
    }
}
