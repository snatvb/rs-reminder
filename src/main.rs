extern crate dotenv;
use std::env;
use teloxide::{prelude::*, utils::command::BotCommands};
// #[macro_use]
// extern crate dotenv_codegen;

#[derive(BotCommands, Clone)]
#[command(
    rename_rule = "lowercase",
    description = "These commands are supported:"
)]
enum Command {
    #[command(description = "display this text.")]
    Help,
}

async fn answer(bot: Bot, msg: Message, cmd: Command) -> ResponseResult<()> {
    match cmd {
        Command::Help => {
            bot.send_message(msg.chat.id, Command::descriptions().to_string())
                .await?
        }
    };

    Ok(())
}

#[tokio::main]
async fn main() {
    load_env();

    pretty_env_logger::init();
    log::info!("Starting command bot...");

    let teloxide_token = env::var("TELOXIDE_TOKEN").expect("TELOXIDE_TOKEN must be set.");
    log::info!("Starting throw dice bot...");

    let bot = Bot::new(teloxide_token);

    Command::repl(bot, answer).await;
}

fn load_env() {
    dotenv::dotenv().ok();
    #[cfg(debug_assertions)]
    dotenv::from_filename(".env.debug").ok();
}
