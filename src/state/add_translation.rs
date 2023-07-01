use async_trait::async_trait;
use teloxide::{payloads::SendMessageSetters, requests::Requester, types::Message};

use crate::keyboard;

use super::{error::StateResult, idle, State};

#[derive(Debug, Clone)]
pub struct AddTranslation {
    word: String,
}

impl AddTranslation {
    pub fn new(word: &str) -> AddTranslation {
        AddTranslation {
            word: word.to_owned(),
        }
    }
}

#[async_trait]
impl State for AddTranslation {
    async fn handle_message(
        &self,
        ctx: &super::Context,
        msg: Message,
    ) -> StateResult<Box<dyn State>> {
        if let Some(text) = msg.text() {
            let word = ctx.db.new_word(ctx.chat_id.0, &self.word, text).await?;
            ctx.bot
                .send_message(
                    msg.chat.id,
                    format!(
                        "Good {} with translation {} has been added",
                        word.word, word.translate
                    ),
                )
                .reply_markup(keyboard::Button::Cancel.to_keyboard())
                .await?;
            return Ok(Box::new(idle::Idle::new()));
        }

        Ok(self.clone_state())
    }

    fn clone_state(&self) -> Box<dyn State> {
        Box::new(self.clone())
    }
}
