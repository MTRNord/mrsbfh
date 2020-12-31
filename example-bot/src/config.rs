use mrsbfh::config::ConfigDerive;
use serde::{Deserialize, Serialize};
use std::borrow::Cow;

#[derive(Debug, PartialEq, Serialize, Deserialize, Clone, ConfigDerive)]
pub struct Config<'a> {
    pub homeserver_url: Cow<'a, str>,
    pub mxid: Cow<'a, str>,
    pub password: Cow<'a, str>,
    pub store_path: Cow<'a, str>,
    pub session_path: Cow<'a, str>,
}
