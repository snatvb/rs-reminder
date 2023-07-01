use async_trait::async_trait;
use teloxide::{requests::Requester, types::Message};

use super::{add_translation, error::StateResult, State};

#[derive(Clone, Debug)]
pub struct AddWord {}

impl AddWord {
    pub fn new() -> AddWord {
        AddWord {}
    }
}

#[async_trait]
impl State for AddWord {
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
            ctx.bot
                .send_message(msg.chat.id, "Enter translation")
                .await?;
            return Ok(Box::new(add_translation::AddTranslation::new(&word)));
        }

        Ok(self.clone_state())
    }

    fn clone_state(&self) -> Box<dyn State> {
        Box::new(self.clone())
    }
}
