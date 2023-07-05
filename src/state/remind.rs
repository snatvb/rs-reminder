use async_trait::async_trait;
use teloxide::{payloads::SendMessageSetters, requests::Requester, types::ChatId};

use crate::prisma;

use super::{error::StateResult, State};

#[derive(Debug, Clone)]
pub struct Remind {
    word: prisma::word::Data,
}

impl Remind {
    pub fn new(word: prisma::word::Data) -> Remind {
        Remind { word }
    }
}

#[async_trait]
impl State for Remind {
    async fn on_enter(&self, ctx: &super::Context, _: Option<Box<dyn State>>) -> StateResult<()> {
        ctx.bot
            .send_message(
                ChatId(self.word.chat_id),
                format!("Write translation for the word `{}`", self.word.word),
            )
            .parse_mode(teloxide::types::ParseMode::MarkdownV2)
            .await?;
        Ok(())
    }

    fn clone_state(&self) -> Box<dyn State> {
        Box::new(self.clone())
    }
}
