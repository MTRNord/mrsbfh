//! # Utils
//!
//! ## Session
//!
//! The easiest way to use the [Session](crate::utils::Session) struct is to use it like this:
//!
//! ```compile_fail
//! use matrix_sdk::Session as SDKSession;
//!
//! if let Some(session) = Session::load(config.session_path.parse().unwrap()) {
//!     info!("Starting relogin");
//!
//!     let session = SDKSession {
//!         access_token: session.access_token,
//!         device_id: session.device_id.into(),
//!         user_id: matrix_sdk::identifiers::UserId::try_from(session.user_id.as_str()).unwrap(),
//!     };
//!
//!     if let Err(e) = client.restore_login(session).await {
//!         error!("{}", e);
//!     };
//!     info!("Finished relogin");
//! } else {
//!     info!("Starting login");
//!     let login_response = client
//!         .login(
//!             &config.mxid,
//!             &config.password,
//!             None,
//!             Some(&"timetracking-bot".to_string()),
//!         )
//!         .await;
//!     match login_response {
//!         Ok(login_response) => {
//!            info!("Session: {:#?}", login_response);
//!            let session = Session {
//!                 homeserver: client.homeserver().to_string(),
//!                 user_id: login_response.user_id.to_string(),
//!                 access_token: login_response.access_token,
//!                 device_id: login_response.device_id.into(),
//!             };
//!             session.save(config.session_path.parse().unwrap())?;
//!         }
//!         Err(e) => error!("Error while login: {}", e),
//!     }
//!     info!("Finished login");
//! }
//! ```
//!
//! <br>
//!
//! This first checks if there is a session already existing and uses it to relogin using the known
//! session data.
//!
//! If not it creates and saves the [Session](crate::utils::Session) struct. Allowing for a relogin on the next start.
//!
//! ## Autojoin
//!
//! The [`#[autojoin]`](crate::utils::autojoin) macro is used to generate the logic in the
//! [EventEmitter](matrix_sdk::EventEmitter) to handle invites for the bot. It is executed after your code.
//!
//! The usage is:
//!
//! ```compile_fail
//! use matrix_sdk::async_trait;
//! use matrix_sdk::{
//!     events::{
//!         room::message::MessageEventContent, StrippedStateEvent,
//!     },
//!     EventEmitter, SyncRoom, Client
//! };
//! use tracing::*;
//!
//! #[derive(Debug, Clone)]
//! pub struct Bot {
//!     client: Client,
//! }
//!
//! #[mrsbfh::utils::autojoin]
//! #[async_trait]
//! impl EventEmitter for Bot {
//!      async fn on_stripped_state_member(
//!         &self,
//!         room: SyncRoom,
//!         room_member: &StrippedStateEvent<MemberEventContent>,
//!         _: Option<MemberEventContent>,
//!     ) {
//!         println!("autojoin example")
//!     }
//! }
//! ```
//!
//! <br>
//!
//! **This does have some requirements:**
//! * The struct MUST have a field `client` which is the [MatrixSDK Client](matrix_sdk::Client)
//! * The `#[async_trait]` macro MUST be below the `#[autojoin]` macro
//! * The `on_stripped_state_member` method MUST exist and the arguments MUST be named the way they are named
//! in the example.
//!

use serde::{Deserialize, Serialize};
use std::error::Error;
use std::path::PathBuf;
use tracing::*;
use crate::errors::SessionError;

pub use mrsbfh_macros::autojoin;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Session {
    /// The homeserver used for this session.
    pub homeserver: String,
    /// The access token used for this session.
    pub access_token: String,
    /// The user the access token was issued for.
    pub user_id: String,
    /// The ID of the client device
    pub device_id: String,
}

impl Session {
    pub fn save(&self, session_path: PathBuf) -> Result<(), SessionError> {
        let mut session_path: PathBuf = session_path;
        info!("SessionPath: {:?}", session_path);
        std::fs::create_dir_all(&session_path)?;
        session_path.push("session.json");
        serde_json::to_writer(&std::fs::File::create(session_path)?, self)?;
        Ok(())
    }

    pub fn load(session_path: PathBuf) -> Option<Self> {
        let mut session_path: PathBuf = session_path;
        session_path.push("session.json");
        let file = std::fs::File::open(session_path);
        match file {
            Ok(file) => {
                let session: Result<Self, serde_json::Error> = serde_json::from_reader(&file);
                match session {
                    Ok(session) => Some(session),
                    Err(_) => None,
                }
            }
            Err(_) => None,
        }
    }
}
