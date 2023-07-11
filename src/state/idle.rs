use std::time;

use async_trait::async_trait;
use teloxide::{
    payloads::{EditMessageTextSetters, SendMessageSetters},
    requests::Requester,
    types::CallbackQuery,
};

use crate::{
    keyboard::{self, Button},
    state::remove_words,
};

use super::{
    add_word,
    error::{StateError, StateResult},
    events::Event,
    remind::Remind,
    word_list, State,
};

#[derive(Clone, Debug)]
pub struct Idle {}

impl Idle {
    pub fn new() -> Idle {
        Idle {}
    }
}

impl Idle {
    pub async fn send_start_msg(ctx: &super::Context) -> StateResult<()> {
        log::debug!("Send start message...");
        ctx.bot
            .send_message(ctx.chat_id, "Chose action")
            .reply_markup(keyboard::words_actions())
            .await?;
        log::debug!("SUCCESS: Send start message");

        Ok(())
    }

    async fn handle_cmd(
        &self,
        ctx: &super::Context,
        button: Button,
        query: CallbackQuery,
    ) -> StateResult<Box<dyn State>> {
        let msg = query
            .message
            .ok_or(StateError::ExpectedMessageInsideCallbackQuery)?;
        match button {
            keyboard::Button::AddWord => {
                ctx.bot
                    .edit_message_text(msg.chat.id, msg.id, "Write a word for translation")
                    .reply_markup(keyboard::Button::Cancel.to_keyboard())
                    .await?;
                return Ok(Box::new(add_word::AddWord::new()));
            }
            keyboard::Button::ListWords => {
                return Ok(Box::new(word_list::WordList::new(Some(msg.id), 0)));
            }
            keyboard::Button::RemoveWord => {
                ctx.bot
                    .edit_message_text(msg.chat.id, msg.id, "Write a word for removing")
                    .reply_markup(keyboard::Button::Cancel.to_keyboard())
                    .await?;
                return Ok(Box::new(remove_words::RemoveWords::new()));
            }
            _ => {
                return Err(StateError::UnexpectedCommand(format!(
                    "Unexpected command: {} - {}",
                    button.key(),
                    button.text()
                )));
            }
        }
    }
}

#[async_trait]
impl State for Idle {
    async fn on_enter(
        &self,
        ctx: &super::Context,
        from: Option<Box<dyn State>>,
    ) -> StateResult<()> {
        log::debug!(
            "Entered {} state from {:?}",
            self.name(),
            from.clone().map(|s| s.name())
        );
        if from.is_some() {
            Idle::send_start_msg(ctx).await?;
        }
        Ok(())
    }

    fn timeout(&self) -> Option<time::Duration> {
        None
    }

    async fn handle_event(
        &self,
        ctx: &super::Context,
        event: Event,
    ) -> StateResult<Box<dyn State>> {
        match event {
            Event::RemindWordToUser(word, user) => Ok(Box::new(Remind::new(word, user))),
            Event::Button(button, query) => self.handle_cmd(ctx, button, query).await,
            _ => Ok(self.clone_state()),
        }
    }

    fn clone_state(&self) -> Box<dyn State> {
        Box::new(self.clone())
    }
}
