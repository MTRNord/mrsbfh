//! # Commands
//!
//! ## `#[command]` macro
//!
//! Commands are defined in their own submodules using a function which name defines the command name.
//!
//! These functions require a specific syntax which is described below.
//!
//! Also that function requires you to have a config struct which implements the [Config](crate::config::Config)
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
//! use matrix_sdk::events::{room::message::MessageEventContent, AnyMessageEventContent};
//! use mrsbfh::commands::command;
//! use mrsbfh::config::Config;
//! use std::error::Error;
//!
//! #[command(help = "`!hello_world` - Prints \"hello world\".")]
//! pub async fn hello_world<C: Config>(
//!     tx: mrsbfh::Sender,
//!     _config: C,
//!     _sender: String,
//!     mut _args: Vec<&str>,
//! ) -> Result<(), Box<dyn Error>> {
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
//!     HelloWorld
//! }
//! ```
//!
//! This does generate a `match_command` function which takes the following arguments:
//!
//! `(command: &str, config: C, tx: mrsbfh::Sender, sender: String, args: Vec<&str>)`
//!
//! and it returns: `Result<(), Box<dyn Error>>`.
//!
//! This can either be called by you or you can continue reading and instead use another macro to
//! do this for you.
//!
//! <br>
//!
//! ## `#[commands]` macro
//!
//! This macro is used to generate the logic in the [EventEmitter](matrix_sdk::EventEmitter) to
//! handle commands after your code.
//!
//! The usage is:
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
//!     Client, EventEmitter, SyncRoom,
//! };
//! use tracing::*;
//!
//! #[derive(Debug, Clone)]
//! pub struct Bot {
//!     client: Client,
//!     config: Config<'static>,
//! }
//!
//! #[mrsbfh::commands::commands]
//! #[async_trait]
//! impl EventEmitter for Bot {
//!     async fn on_room_message(&self, room: SyncRoom, event: &SyncMessageEvent<MessageEventContent>) {
//!         println!("message example")
//!     }
//! }
//! ```
//!
//! <br>
//!
//! **This does have some requirements:**
//! * Your `match_command` function MUST be imported
//! * The struct MUST have a field `config` which implements the [Config](crate::config::Config)
//! trait
//! * The struct MUST have a field `client` which is the [MatrixSDK Client](matrix_sdk::Client)
//! * The `#[async_trait]` macro MUST be below the `#[commands]` macro
//! * The `on_room_message` method MUST exist and the arguments MUST be named the way they are named
//! in the example.
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

#[cfg(feature = "macros")]
pub use mrsbfh_macros::{command, command_generate, commands};
