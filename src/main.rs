extern crate dotenv;
mod clients;
mod common;
#[allow(warnings)]
mod prisma;
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
async fn main() {
    println!("Starting bot...");
    load_env();

    pretty_env_logger::init();

    let db = prisma::PrismaClient::_builder()
        .build()
        .await
        .expect("Failed to connect to database");
    let teloxide_token = env::var("TELOXIDE_TOKEN").expect("TELOXIDE_TOKEN must be set.");
    let last5 = &teloxide_token[teloxide_token.len() - 5..];
    log::info!("Starting throw dice bot with token {}...", last5);

    let bot = Bot::new(teloxide_token);

    let handler = dptree::entry()
        .branch(Update::filter_message().endpoint(handle_message))
        .branch(Update::filter_callback_query().endpoint(callback_handler))
        .branch(Update::filter_inline_query().endpoint(inline_query_handler));

    Dispatcher::builder(bot.clone(), handler)
        .dependencies(dptree::deps![clients::Clients::new(bot, Storage::new(db))])
        .enable_ctrlc_handler()
        .build()
        .dispatch()
        .await;
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

/// When it receives a callback from a button it edits the message with all
/// those buttons writing a text with the selected Debian version.
///
/// **IMPORTANT**: do not send privacy-sensitive data this way!!!
/// Anyone can read data stored in the callback button.
// async fn callback_handler(bot: Bot, q: CallbackQuery) -> ResponseResult<()> {
//     if let Some(version) = q.data {
//         let text = format!("You chose: {version}");

//         // Tell telegram that we've seen this query, to remove 🕑 icons from the
//         // clients. You could also use `answer_callback_query`'s optional
//         // parameters to tweak what happens on the client side.
//         bot.answer_callback_query(q.id).await?;

//         // Edit text of the message to which the buttons were attached
//         if let Some(Message { id, chat, .. }) = q.message {
//             bot.edit_message_text(chat.id, id, text).await?;
//         } else if let Some(id) = q.inline_message_id {
//             bot.edit_message_text_inline(id, text).await?;
//         }

//         log::info!("You chose: {}", version);
//     }

//     Ok(())
// }
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

// async fn handle_message(bot: Bot, msg: Message, me: Me) -> ResponseResult<()> {
//     if let Some(text) = msg.text() {
//         log::info!("Got text message: {}", text);
//         match BotCommands::parse(text, me.username()) {
//             Ok(common::Command::Help) => {
//                 // Just send the description of all commands.
//                 bot.send_message(msg.chat.id, common::Command::descriptions().to_string())
//                     .await?;
//             }
//             Ok(common::Command::Start) => {
//                 // Create a list of buttons and send them.
//                 let keyboard = make_keyboard();
//                 bot.send_message(msg.chat.id, "Debian versions:")
//                     .reply_markup(keyboard)
//                     .await?;
//             }

//             Err(_) => {
//                 bot.send_message(msg.chat.id, "Command not found!").await?;
//             }
//         }
//     }

//     Ok(())
// }

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
