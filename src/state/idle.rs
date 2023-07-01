use async_trait::async_trait;
use teloxide::{
    payloads::{EditMessageTextSetters, SendMessageSetters},
    requests::{Requester, ResponseResult},
    types::Message,
    utils::command::BotCommands,
};

use crate::{common::Command, keyboard};

use super::{add_word, error::StateResult, State};

#[derive(Clone, Debug)]
pub struct Idle {}

impl Idle {
    pub fn new() -> Idle {
        Idle {}
    }
}

impl Idle {
    async fn send_start_msg(&self, ctx: &super::Context) -> ResponseResult<()> {
        ctx.bot
            .send_message(ctx.chat_id, "Chose action")
            .reply_markup(keyboard::words_actions())
            .await?;

        Ok(())
    }
}

#[async_trait]
impl State for Idle {
    async fn on_enter(
        &self,
        ctx: &super::Context,
        from: Option<Box<dyn State>>,
    ) -> StateResult<()> {
        log::debug!("Entered {} state", self.name());
        if let Some(from) = from {
            log::debug!("From: {}", from.name());
            self.send_start_msg(ctx).await?;
        }

        Ok(())
    }

    async fn handle_message(
        &self,
        ctx: &super::Context,
        msg: Message,
    ) -> StateResult<Box<dyn State>> {
        let me = ctx.bot.get_me().await?;
        if let Some(text) = msg.text() {
            match BotCommands::parse(text, me.username()) {
                Ok(Command::Help) => {
                    // Just send the description of all commands.
                    ctx.bot
                        .send_message(msg.chat.id, Command::descriptions().to_string())
                        .await?;
                }

                Ok(Command::Start) => {
                    self.send_start_msg(ctx).await?;
                }

                Err(_) => {
                    ctx.bot
                        .send_message(msg.chat.id, "Command not found!")
                        .await?;
                }
            }
        }

        Ok(self.clone_state())
    }

    async fn handle_callback_query(
        &self,
        ctx: &super::Context,
        query: teloxide::types::CallbackQuery,
    ) -> StateResult<Box<dyn State>> {
        log::info!("Callback query in IDLE: {:?}", query.data);
        if let Some(button) = query.data {
            if let Some(Message { id, chat, .. }) = query.message {
                ctx.bot
                    .edit_message_text(chat.id, id, "Write a word for translation")
                    .reply_markup(keyboard::Button::Cancel.to_keyboard())
                    .await?;
            }

            if button == "add_word" {
                return Ok(Box::new(add_word::AddWord::new()));
            }
        }
        Ok(self.clone_state())
    }

    fn clone_state(&self) -> Box<dyn State> {
        Box::new(self.clone())
    }
}
