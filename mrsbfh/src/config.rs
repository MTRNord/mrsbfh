//! # Config
//!
//! The Config Trait is used to be able to pass your Config to the commands as well as use them for
//! the path of the session.
//!
//! <br>
//!
//! The simplest way is to use the Derive macro [`#[derive(ConfigDerive)]`](crate::config::ConfigDerive)
//!
//! It requires however that You also derive [Clone](std::clone::Clone), [Serialize](serde::Serialize)
//! and [Deserialize](serde::Deserialize).
//!
//! Also this only works for yaml config files. For any other format you will to implement the trait
//! yourself. However the crate still needs to implement [Clone](std::clone::Clone).
//!
//! ## Example
//!
//! ```compile_fail
//! use serde::{Deserialize, Serialize};
//! use std::borrow::Cow;
//!
//! #[derive(Debug, PartialEq, Serialize, Deserialize, Clone, ConfigDerive)]
//! pub struct Config<'a> {
//!     pub homeserver_url: Cow<'a, str>,
//!     pub mxid: Cow<'a, str>,
//!     pub password: Cow<'a, str>,
//!     pub store_path: Cow<'a, str>,
//!     pub session_path: Cow<'a, str>,
//! }
//! ```
//!

use serde::de::DeserializeOwned;
use serde::Serialize;
use std::error::Error;
use std::path::Path;

pub use mrsbfh_macros::ConfigDerive;

pub trait Config {
    fn load<P: AsRef<Path> + std::fmt::Debug>(path: P) -> Result<Self, Box<dyn Error>>
    where
        Self: Sized + Serialize + DeserializeOwned;
}
