pub mod config;
pub mod translation;
use teloxide::macros::BotCommands;

pub type AsyncMutex<T> = tokio::sync::Mutex<T>;

#[derive(BotCommands, Clone, Debug)]
#[command(
    rename_rule = "lowercase",
    description = "These commands are supported:"
)]
pub enum Command {
    #[command(description = "display this text.")]
    Help,
    #[command(description = "Start")]
    Start,
}
