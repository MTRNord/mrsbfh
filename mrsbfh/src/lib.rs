//! [![github]](https://github.com/MTRNord/mrsbfh)&ensp;[![crates-io]](https://crates.io/crates/mrsbfh)&ensp;[![docs-rs]](crate)
//!
//! [github]: https://img.shields.io/badge/github-8da0cb?style=for-the-badge&labelColor=555555&logo=github
//! [crates-io]: https://img.shields.io/badge/crates.io-fc8d62?style=for-the-badge&labelColor=555555&logo=rust
//! [docs-rs]: https://img.shields.io/badge/docs.rs-66c2a5?style=for-the-badge&labelColor=555555&logoColor=white&logo=data:image/svg+xml;base64,PHN2ZyByb2xlPSJpbWciIHhtbG5zPSJodHRwOi8vd3d3LnczLm9yZy8yMDAwL3N2ZyIgdmlld0JveD0iMCAwIDUxMiA1MTIiPjxwYXRoIGZpbGw9IiNmNWY1ZjUiIGQ9Ik00ODguNiAyNTAuMkwzOTIgMjE0VjEwNS41YzAtMTUtOS4zLTI4LjQtMjMuNC0zMy43bC0xMDAtMzcuNWMtOC4xLTMuMS0xNy4xLTMuMS0yNS4zIDBsLTEwMCAzNy41Yy0xNC4xIDUuMy0yMy40IDE4LjctMjMuNCAzMy43VjIxNGwtOTYuNiAzNi4yQzkuMyAyNTUuNSAwIDI2OC45IDAgMjgzLjlWMzk0YzAgMTMuNiA3LjcgMjYuMSAxOS45IDMyLjJsMTAwIDUwYzEwLjEgNS4xIDIyLjEgNS4xIDMyLjIgMGwxMDMuOS01MiAxMDMuOSA1MmMxMC4xIDUuMSAyMi4xIDUuMSAzMi4yIDBsMTAwLTUwYzEyLjItNi4xIDE5LjktMTguNiAxOS45LTMyLjJWMjgzLjljMC0xNS05LjMtMjguNC0yMy40LTMzLjd6TTM1OCAyMTQuOGwtODUgMzEuOXYtNjguMmw4NS0zN3Y3My4zek0xNTQgMTA0LjFsMTAyLTM4LjIgMTAyIDM4LjJ2LjZsLTEwMiA0MS40LTEwMi00MS40di0uNnptODQgMjkxLjFsLTg1IDQyLjV2LTc5LjFsODUtMzguOHY3NS40em0wLTExMmwtMTAyIDQxLjQtMTAyLTQxLjR2LS42bDEwMi0zOC4yIDEwMiAzOC4ydi42em0yNDAgMTEybC04NSA0Mi41di03OS4xbDg1LTM4Ljh2NzUuNHptMC0xMTJsLTEwMiA0MS40LTEwMi00MS40di0uNmwxMDItMzguMiAxMDIgMzguMnYuNnoiPjwvcGF0aD48L3N2Zz4K
//!
//! <br>
//!
//! # MRSBFH - Matrix-Rust-SDK-Bot-Framework-Helper
//!
//! `mrsbfh` is a collection of utilities to make performing certain tasks in command bots
//! with matrix easier.
//!
//! ## Features
//!
//! * Macro for simple autojoin functionality
//! * Macros for pretty defining of commands
//! * Utils for a simple Config
//! * Utils for restoring ad saving matrix sessions
//!
//! ## Examples
//!
//! For examples please have a look at the [example-bot](https://github.com/MTRNord/mrsbfh/tree/main/example-bot) or take a look in the individual modules.

pub mod commands;
pub mod config;
pub mod errors;
pub mod sync;
pub mod utils;

pub type Sender = tokio::sync::mpsc::Sender<matrix_sdk::ruma::events::AnyMessageEventContent>;

#[async_trait::async_trait]
pub trait MatrixMessageExt {
    async fn send_notice(
        &mut self,
        body: String,
        formatted_body: Option<String>,
    ) -> Result<
        (),
        tokio::sync::mpsc::error::SendError<matrix_sdk::ruma::events::AnyMessageEventContent>,
    >;
}

#[async_trait::async_trait]
impl MatrixMessageExt for Sender {
    async fn send_notice(
        &mut self,
        body: String,
        formatted_body: Option<String>,
    ) -> Result<
        (),
        tokio::sync::mpsc::error::SendError<matrix_sdk::ruma::events::AnyMessageEventContent>,
    > {
        match formatted_body {
            Some(formatted_body) => {
                let content = matrix_sdk::ruma::events::AnyMessageEventContent::RoomMessage(
                    matrix_sdk::ruma::events::room::message::MessageEventContent::notice_html(
                        body,
                        formatted_body,
                    ),
                );

                self.send(content).await
            }
            None => {
                let content = matrix_sdk::ruma::events::AnyMessageEventContent::RoomMessage(
                    matrix_sdk::ruma::events::room::message::MessageEventContent::notice_plain(
                        body,
                    ),
                );
                self.send(content).await
            }
        }
    }
}

pub use pulldown_cmark;
pub use serde_yaml;
pub use tokio;
pub use tracing;
pub use url;
