//! # Various small Utils
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

use crate::errors::SessionError;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use tracing::*;

/// Informations needed to keep track about a session
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
    /// Save the session to the specified path
    pub fn save(&self, session_path: PathBuf) -> Result<(), SessionError> {
        let mut session_path: PathBuf = session_path;
        debug!("SessionPath: {:?}", session_path);
        std::fs::create_dir_all(&session_path)?;
        session_path.push("session.json");
        serde_json::to_writer(&std::fs::File::create(session_path)?, self)?;
        Ok(())
    }

    /// Load the session from a specified path
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
