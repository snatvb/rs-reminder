use async_trait::async_trait;
use chrono::prelude::*;
use teloxide::{requests::Requester, types::Message};

use crate::common::config::TIMINGS;

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
            let translation = text.to_owned();
            let chat_id: i64 = ctx.chat_id.0;
            let now = Utc::now();
            let first_remind =
                now + chrono::Duration::hours(TIMINGS.get(&0i32).unwrap().to_owned());
            let first_remind = first_remind.with_timezone(&FixedOffset::east_opt(0).unwrap());
            let word = ctx
                .db
                .word()
                .create(
                    chat_id,
                    self.word.clone(),
                    translation,
                    first_remind,
                    vec![],
                )
                .exec()
                .await?;
            ctx.bot
                .send_message(
                    msg.chat.id,
                    format!(
                        "Good {} with translation {} has been added",
                        word.word, word.translate
                    ),
                )
                .await?;
            return Ok(Box::new(idle::Idle::new()));
        }

        Ok(self.clone_state())
    }

    fn clone_state(&self) -> Box<dyn State> {
        Box::new(self.clone())
    }
}
