use async_trait::async_trait;
use chrono::prelude::*;
use teloxide::{
    requests::{Requester, ResponseResult},
    types::Message,
};

use super::{idle, State};

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
    ) -> ResponseResult<Box<dyn State>> {
        if let Some(text) = msg.text() {
            let translation = text.to_owned();
            ctx.bot
                .send_message(
                    msg.chat.id,
                    format!(
                        "Good {} with translation {} has been added",
                        self.word, translation
                    ),
                )
                .await?;
            let chat_id: i64 = ctx.chat_id.0;
            let now = Utc::now();
            let in_an_hour = now + chrono::Duration::hours(1);
            let in_an_hour = in_an_hour.with_timezone(&FixedOffset::east_opt(0).unwrap());
            ctx.db
                .word()
                .create(chat_id, self.word.clone(), translation, in_an_hour, vec![])
                .exec()
                .await
                .unwrap();
            return Ok(Box::new(idle::Idle::new()));
        }

        Ok(self.clone_state())
    }

    fn clone_state(&self) -> Box<dyn State> {
        Box::new(self.clone())
    }
}
