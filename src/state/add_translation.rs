use async_trait::async_trait;
use teloxide::{payloads::SendMessageSetters, requests::Requester, types::Message};

use crate::common::translation;

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
            let translation = translation::Translation::from(text);
            let translations = translation.to_string();
            if translation.is_empty() {
                ctx.bot
                    .send_message(
                        msg.chat.id,
                        format!("Translation for {} is empty", self.word),
                    )
                    .await?;
                return Ok(self.clone_state());
            }

            let word = ctx
                .db
                .new_word(ctx.chat_id.0, &self.word, &translations)
                .await?;

            ctx.bot
                .send_message(
                    msg.chat.id,
                    format!(
                        "Word {} has been added\\.\n\nTranslations: {}",
                        word.word,
                        translation.to_formatted_string()
                    ),
                )
                .parse_mode(teloxide::types::ParseMode::MarkdownV2)
                .await?;
            return Ok(Box::new(idle::Idle::new()));
        }

        Ok(self.clone_state())
    }

    fn clone_state(&self) -> Box<dyn State> {
        Box::new(self.clone())
    }
}
