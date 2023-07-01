use async_trait::async_trait;
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
            return Ok(Box::new(idle::Idle::new()));
        }

        Ok(self.clone_state())
    }

    fn clone_state(&self) -> Box<dyn State> {
        Box::new(self.clone())
    }
}
