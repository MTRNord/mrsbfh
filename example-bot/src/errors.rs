use matrix_sdk::ruma::events::room::message::RoomMessageEventContent;
use thiserror::Error as ThisError;

#[derive(ThisError, Debug)]
pub enum Error {
    #[error(transparent)]
    SendError(#[from] tokio::sync::mpsc::error::SendError<RoomMessageEventContent>),
}
