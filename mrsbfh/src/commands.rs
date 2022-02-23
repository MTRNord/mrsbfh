//! # Helpers to construct Commands
//!
//! ## `#[command]` macro
//!
//! Commands are defined in their own submodules using a function which name defines the command name.
//!
//! These functions require a specific syntax which is described below.
//!
//! Also that function requires you to have a config struct which implements the [Loader](crate::config::Loader)
//! trait.
//!
//! <br>
//!
//! To use the commands macros you want a module where these commands are stored in. This is because
//! each is REQUIRED to be in a submodule with the SAME name as the commands name.
//!
//! In each of these submodules you can define a command like this:
//!
//! ```compile_fail
//! use crate::config::Config;
//! use crate::errors::Error;
//! use matrix_sdk::ruma::events::{room::message::MessageEventContent, AnyMessageEventContent};
//! use matrix_sdk::ruma::RoomId;
//! use matrix_sdk::Client;
//! use mrsbfh::commands::command;
//! use std::sync::Arc;
//! use tokio::sync::Mutex;
//!
//! #[command(help = "`!hello_world` - Prints \"hello world\".")]
//! pub async fn hello_world<'a>(
//!     _client: Client,
//!     tx: mrsbfh::Sender,
//!     _config: Arc<Mutex<Config<'a>>>,
//!     _sender: String,
//!     _room_id: RoomId,
//!     mut _args: Vec<&str>,
//! ) -> Result<(), Error>
//! where
//!     Config<'a>: mrsbfh::config::Loader + Clone,
//! {
//!     let content =
//!         AnyMessageEventContent::RoomMessage(MessageEventContent::notice_plain("Hello World!"));
//!
//!     tx.send(content).await?;
//!     Ok(())
//! }
//! ```
//!
//! <br>
//!
//! ## `#[command_generate]` macro
//!
//! You can now either build your own match statement and help or to make this more convenient you
//! can use the `command_generate` macro to do this for you.
//!
//! To use it you are required to have the following code structure inside the file that is the
//! direct parent module of the commands.
//!
//! ```compile_fail
//! use mrsbfh::commands::command_generate;
//!
//! pub mod hello_world;
//!
//! #[command_generate(
//!     bot_name = "Example",
//!     description = "This bot prints hello!"
//! )]
//! enum Commands {
//!     Hello_World
//! }
//! ```
//!
//! This does generate a `match_command` function which takes the following arguments:
//!
//! `(client: Client, tx: mrsbfh::Sender, config: Arc<Mutex<Config<'a>>>, sender: String, room_id: RoomId, args: Vec<&str>)`
//!
//! and it returns: `Result<(), Error>` where Error is an Error struct you provide.
//!
//! This can either be called by you or you can continue reading and instead use another macro to
//! do this for you.
//!
//! <br>
//!
//! ## `#[commands]` macro
//!
//! This macro is used to generate the logic in the [register_event_handler](matrix_sdk::Client::register_event_handler) method to
//! handle commands after your code.
//!
//! The definition is:
//!
//! ```compile_fail
//! use crate::commands::match_command;
//! use crate::config::Config;
//! use matrix_sdk::async_trait;
//! use matrix_sdk::{
//!     events::{
//!         room::message::MessageEventContent,
//!         SyncMessageEvent,
//!     },
//!     Client, SyncRoom,
//! };
//! use tracing::*;
//!
//!
//! #[mrsbfh::commands::commands]
//! pub(crate) async fn on_room_message(
//!     event: SyncMessageEvent<MessageEventContent>,
//!     room: Room,
//!     client: Client,
//!     config: Arc<Mutex<Config<'static>>>,
//! ) {
//!     println!("message example")
//! }
//! ```
//!
//! You use it using this snippet:
//!
//! ```
//! client
//!     .register_event_handler(move |ev, room, client| {
//!         sync::on_room_message(ev, room, client, config.clone())
//!     })
//!     .await;
//! ```
//!
//! <br>
//!
//! **This does have some requirements:**
//! * Your `match_command` function MUST be imported
//!

pub mod command_utils {
    use lazy_static::lazy_static;

    lazy_static! {
        pub static ref WHITESPACE_DEDUPLICATOR_MAGIC: regex::Regex =
            regex::Regex::new(r"\s+").unwrap();
        pub static ref COMMAND_MATCHER_MAGIC: regex::Regex =
            regex::Regex::new(r"!([\w-]+)").unwrap();
    }
}

pub use mrsbfh_macros::{command, command_generate, commands};
