use crate::config::Config;
use crate::errors::Error;
use mrsbfh::commands::command_generate;

pub mod hello_world;

#[command_generate(bot_name = "Example", description = "This bot prints hello!")]
enum Commands {
    HelloWorld,
}
