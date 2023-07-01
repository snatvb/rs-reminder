use async_trait::async_trait;

use super::{add_word, State};

#[derive(Clone, Debug)]
pub struct Idle {}

impl Idle {
    pub fn new() -> Idle {
        Idle {}
    }
}

#[async_trait]
impl State for Idle {
    fn clone_state(&self) -> Box<dyn State> {
        Box::new(self.clone())
    }

    async fn handle_callback_query(
        &self,
        _: super::Context,
        query: teloxide::types::CallbackQuery,
    ) -> Result<Box<dyn State>, Box<dyn std::error::Error + Send + Sync>> {
        if let Some(button) = query.data {
            if button == "add_word" {
                return Ok(Box::new(add_word::AddWord::new()));
            }
        }
        Ok(self.clone_state())
    }
}
