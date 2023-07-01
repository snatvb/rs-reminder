use async_trait::async_trait;
use teloxide::{
    payloads::SendMessageSetters,
    requests::Requester,
    types::{Message, ParseMode},
};

use super::{error::StateResult, State};

#[derive(Clone, Debug, Default)]
pub struct RemoveWords {}

impl RemoveWords {
    pub fn new() -> RemoveWords {
        RemoveWords {}
    }
}

#[async_trait]
impl State for RemoveWords {
    async fn on_enter(&self, _: &super::Context, _: Option<Box<dyn State>>) -> StateResult<()> {
        log::info!("Entered AddWord state");

        Ok(())
    }

    async fn handle_message(
        &self,
        ctx: &super::Context,
        msg: Message,
    ) -> StateResult<Box<dyn State>> {
        if let Some(text) = msg.text() {
            let word = text.to_owned();
            let _ = ctx.db.remove_word(ctx.chat_id.0, &word).await?;
            let response = format!(
                "Word `{}` has been removed\nWrite more or write /start to back to main menu",
                word
            );
            ctx.bot
                .send_message(msg.chat.id, response)
                .parse_mode(ParseMode::MarkdownV2)
                .await?;
            return Ok(self.clone_state());
        }

        Ok(self.clone_state())
    }

    fn clone_state(&self) -> Box<dyn State> {
        Box::new(self.clone())
    }
}
