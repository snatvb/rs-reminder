use async_trait::async_trait;
use teloxide::{
    payloads::{EditMessageTextSetters, SendMessageSetters},
    requests::Requester,
    types::Message,
};

use crate::{keyboard, state::remove_words};

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
    async fn send_start_msg(&self, ctx: &super::Context) -> StateResult<()> {
        ctx.bot
            .send_message(ctx.chat_id, "Chose action")
            .reply_markup(keyboard::words_actions())
            .await?;

        Ok(())
    }
}

#[async_trait]
impl State for Idle {
    async fn on_enter(&self, ctx: &super::Context, _: Option<Box<dyn State>>) -> StateResult<()> {
        log::debug!("Entered {} state", self.name());
        self.send_start_msg(ctx).await?;
        Ok(())
    }

    async fn handle_callback_query(
        &self,
        ctx: &super::Context,
        query: teloxide::types::CallbackQuery,
    ) -> StateResult<Box<dyn State>> {
        log::info!("Callback query in {}: {:?}", self.name(), query.data);
        let request = (keyboard::Button::from_option_key(query.data), query.message);
        if let (Ok(button), Some(Message { id, chat, .. })) = request {
            match button {
                keyboard::Button::AddWord => {
                    ctx.bot
                        .edit_message_text(chat.id, id, "Write a word for translation")
                        .reply_markup(keyboard::Button::Cancel.to_keyboard())
                        .await?;
                    return Ok(Box::new(add_word::AddWord::new()));
                }
                keyboard::Button::ListWords => {
                    return Ok(Box::new(word_list::WordList::new(Some(id), 0)));
                }
                keyboard::Button::RemoveWord => {
                    ctx.bot
                        .edit_message_text(chat.id, id, "Write a word for removing")
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
        Ok(self.clone_state())
    }

    async fn handle_event(&self, _: &super::Context, event: Event) -> StateResult<Box<dyn State>> {
        match event {
            Event::RemindWordToUser(word, user) => Ok(Box::new(Remind::new(word, user))),
            _ => Ok(self.clone_state()),
        }
    }

    fn clone_state(&self) -> Box<dyn State> {
        Box::new(self.clone())
    }
}
