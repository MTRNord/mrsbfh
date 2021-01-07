use serde::{Deserialize, Serialize};
use std::error::Error;
use std::path::PathBuf;
use tracing::*;

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
    pub fn save(&self, session_path: PathBuf) -> Result<(), Box<dyn Error>> {
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
