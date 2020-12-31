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
