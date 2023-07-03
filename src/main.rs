extern crate derive_more;
extern crate dotenv;
mod clients;
mod common;
mod keyboard;
mod macroses;
#[allow(warnings)]
mod prisma;
mod reminder;
mod state;
mod storage;
use std::env;
use teloxide::{
    prelude::*,
    types::{
        InlineKeyboardButton, InlineKeyboardMarkup, InlineQueryResultArticle, InputMessageContent,
        InputMessageContentText,
    },
};

use crate::storage::Storage;

#[tokio::main]
async fn main() -> Result<(), String> {
    println!("Starting bot...");
    load_env();

    pretty_env_logger::init();

    let db = prisma::PrismaClient::_builder()
        .build()
        .await
        .expect("Failed to connect to database");
    let teloxide_token = env::var("TELOXIDE_TOKEN").expect("TELOXIDE_TOKEN must be set.");
    let last5 = &teloxide_token[teloxide_token.len() - 5..];
    log::info!("Starting reminder bot with token {}...", last5);

    let bot = Bot::new(teloxide_token);

    let handler = dptree::entry()
        .branch(Update::filter_message().endpoint(handle_message))
        .branch(Update::filter_callback_query().endpoint(callback_handler))
        .branch(Update::filter_inline_query().endpoint(inline_query_handler));

    let users = clients::Clients::new(bot.clone(), Storage::new(db));
    let mut reminder = reminder::Reminder::new(users.clone());
    let reminder_task = tokio::spawn(async move {
        reminder.run().await;
    });
    let dispatcher_task = tokio::spawn(async move {
        Dispatcher::builder(bot, handler)
            .dependencies(dptree::deps![users])
            .enable_ctrlc_handler()
            .build()
            .dispatch()
            .await;
    });
    tokio::try_join!(reminder_task, dispatcher_task).map_err(|e| e.to_string())?;
    Ok(())
}

async fn inline_query_handler(bot: Bot, q: InlineQuery) -> ResponseResult<()> {
    let choose_debian_version = InlineQueryResultArticle::new(
        "0",
        "Chose debian version",
        InputMessageContent::Text(InputMessageContentText::new("Debian versions:")),
    )
    .reply_markup(make_keyboard());

    bot.answer_inline_query(q.id, vec![choose_debian_version.into()])
        .await?;
    Ok(())
}

async fn callback_handler(
    clients: clients::Clients,
    bot: Bot,
    query: CallbackQuery,
) -> ResponseResult<()> {
    log::info!("Got callback query: {:?}", query.id);
    let id = query.id.to_owned();
    clients.handle_callback_query(query).await?;
    bot.answer_callback_query(id).await?;
    Ok(())
}

async fn handle_message(clients: clients::Clients, message: Message) -> ResponseResult<()> {
    log::info!("Got text message: {}", message.text().unwrap());
    clients.handle_message(message).await?;
    Ok(())
}

fn make_keyboard() -> InlineKeyboardMarkup {
    let mut keyboard: Vec<Vec<InlineKeyboardButton>> = vec![];

    let debian_versions = [
        "Buzz", "Rex", "Bo", "Hamm", "Slink", "Potato", "Woody", "Sarge", "Etch", "Lenny",
        "Squeeze", "Wheezy", "Jessie", "Stretch", "Buster", "Bullseye",
    ];

    for versions in debian_versions.chunks(3) {
        let row = versions
            .iter()
            .map(|&version| InlineKeyboardButton::callback(version.to_owned(), version.to_owned()))
            .collect();

        keyboard.push(row);
    }

    InlineKeyboardMarkup::new(keyboard)
}

fn load_env() {
    dotenv::dotenv().ok();
    #[cfg(debug_assertions)]
    dotenv::from_filename(".env.debug").ok();
}
