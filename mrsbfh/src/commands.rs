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

pub mod extract;

use crate::commands::extract::Extensions;
use async_trait::async_trait;
use std::convert::Infallible;
use std::future::Future;

pub mod command_utils {
    use lazy_static::lazy_static;

    lazy_static! {
        pub static ref WHITESPACE_DEDUPLICATOR_MAGIC: regex::Regex =
            regex::Regex::new(r"\s+").unwrap();
        pub static ref COMMAND_MATCHER_MAGIC: regex::Regex =
            regex::Regex::new(r"!([\w-]+)").unwrap();
    }
}

/// Types that can be created from messages.
///
/// See [`axum::extract`] for more details.
///
/// [`axum::extract`]: https://docs.rs/axum/latest/axum/extract/index.html
#[async_trait]
pub trait FromMessage: Sized {
    /// If the extractor fails it'll use this "rejection" type. A rejection is
    /// a kind of error that can be converted into a response.
    type Rejection: IntoError;

    /// Perform the extraction.
    async fn from_message(msg: &mut MessageParts) -> Result<Self, Self::Rejection>;
}

#[async_trait]
impl FromMessage for Parts {
    type Rejection = Infallible;

    async fn from_message(msg: &mut MessageParts) -> Result<Self, Self::Rejection> {
        let extensions = std::mem::take(msg.extensions_mut());

        let mut temp_message = Message::new();
        *temp_message.extensions_mut() = extensions;

        let parts = temp_message.into_parts();

        Ok(parts)
    }
}

pub struct Parts {
    /// The message's extensions
    pub extensions: Extensions,
}

impl Parts {
    /// Creates a new default instance of `Parts`
    fn new() -> Parts {
        Parts {
            extensions: Extensions::default(),
        }
    }
}

pub struct Message {
    parts: Parts,
}

impl Message {
    /// Creates a new blank `Message`
    ///
    /// The component parts of this message will be set to their default.
    ///
    /// # Examples
    ///
    /// ```
    /// # use msrbfh::commands::Message;
    /// let message = Message::new();
    /// ```
    #[inline]
    pub fn new() -> Message {
        Message {
            parts: Parts::new(),
        }
    }

    /// Creates a new `Message` with the given components parts.
    ///
    /// # Examples
    ///
    /// ```
    /// # use msrbfh::commands::Message;
    /// let message = Message::new();
    /// let mut parts = message.into_parts();
    ///
    /// let message = Message::from_parts(parts);
    /// ```
    #[inline]
    pub fn from_parts(parts: Parts) -> Message {
        Message { parts }
    }

    /// Returns a reference to the associated extensions.
    ///
    /// # Examples
    ///
    /// ```
    /// # use msrbfh::commands::Message;
    /// let message: Message = Message::default();
    /// assert!(message.extensions().get::<i32>().is_none());
    /// ```
    #[inline]
    pub fn extensions(&self) -> &Extensions {
        &self.parts.extensions
    }

    /// Returns a mutable reference to the associated extensions.
    ///
    /// # Examples
    ///
    /// ```
    /// # use msrbfh::commands::Message;
    /// let mut message: Message = Message::default();
    /// message.extensions_mut().insert("hello");
    /// assert_eq!(message.extensions().get(), Some(&"hello"));
    /// ```
    #[inline]
    pub fn extensions_mut(&mut self) -> &mut Extensions {
        &mut self.parts.extensions
    }

    /// Consumes the message returning the parts.
    ///
    /// # Examples
    ///
    /// ```
    /// # use msrbfh::commands::Message;
    /// let message = Message::new(());
    /// let parts = message.into_parts();
    /// ```
    #[inline]
    pub fn into_parts(self) -> Parts {
        self.parts
    }
}

impl Default for Message {
    fn default() -> Self {
        Self::new()
    }
}

pub(crate) mod sealed {
    #![allow(unreachable_pub, missing_docs, missing_debug_implementations)]

    pub trait HiddenTrait {}
    pub struct Hidden;
    impl HiddenTrait for Hidden {}
}

#[derive(Debug)]
pub struct MessageParts {
    extensions: Extensions,
}

impl MessageParts {
    pub fn new(msg: Message) -> Self {
        let Parts { extensions, .. } = msg.into_parts();
        MessageParts { extensions }
    }

    /// Gets a reference to the message extensions.
    pub fn extensions(&self) -> &Extensions {
        &self.extensions
    }

    /// Gets a mutable reference to the message extensions.
    pub fn extensions_mut(&mut self) -> &mut Extensions {
        &mut self.extensions
    }
}

#[async_trait]
pub trait Command<T>: Clone + Send + Sized + 'static {
    // This seals the trait. We cannot use the regular "sealed super trait"
    // approach due to coherence.
    #[doc(hidden)]
    type Sealed: sealed::HiddenTrait;

    /// Call the command with the given request.
    async fn call(self, req: Message) -> Result<(), crate::errors::Errors>;
}

macro_rules! impl_command {
    ( $($ty:ident),* $(,)? ) => {
        #[async_trait]
        #[allow(non_snake_case)]
        impl<F, Fut, Res, $($ty,)*> Command<($($ty,)*)> for F
        where
            F: FnOnce($($ty,)*) -> Fut + Clone + Send + 'static,
            Fut: Future<Output =  Result<(), Res>> + Send,
            Res: IntoError,
            $( $ty: FromMessage + Send,)*
        {
            type Sealed = sealed::Hidden;

            async fn call(self, msg: Message) -> Result<(), crate::errors::Errors> {
                let mut msg = MessageParts::new(msg);

                $(
                    let $ty = match $ty::from_message(&mut msg).await {
                        Ok(value) => value,
                        Err(rejection) => return Err(rejection.into_error()),
                    };
                )*

                let res = self($($ty,)*).await.map_err(|x|x.into_error())?;

                Ok(res)
            }
        }
    };
}

crate::utils::all_the_tuples!(impl_command);

pub use mrsbfh_macros::{command, command_generate, commands};

pub trait IntoError {
    fn into_error(self) -> crate::errors::Errors;
}

impl<T> IntoError for T
where
    T: ToString,
{
    fn into_error(self) -> crate::errors::Errors {
        crate::errors::Errors::CustomError(self.to_string())
    }
}
