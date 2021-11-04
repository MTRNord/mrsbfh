use matrix_sdk::ruma::events::AnyMessageEventContent;
use thiserror::Error as ThisError;

#[derive(ThisError, Debug)]
pub enum Error {
    #[error(transparent)]
    SendError(#[from] tokio::sync::mpsc::error::SendError<AnyMessageEventContent>),
}
