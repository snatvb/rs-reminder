pub mod add_word;
pub mod idle;

use async_trait::async_trait;
use std::{error::Error, fmt::Debug};

use teloxide::{
    requests::Requester,
    types::{CallbackQuery, ChatId, InlineQuery, Message},
};

#[derive(Clone, Debug)]
pub struct FSM {
    pub state: Box<dyn State>,
    pub context: Context,
}

impl FSM {
    pub fn new(state: Box<dyn State>, context: Context) -> Self {
        Self { state, context }
    }

    pub async fn handle_message(&mut self, msg: Message) {
        let handled = self.state.handle_message(&self.context.clone(), msg).await;
        self.state = match handled {
            Ok(state) => state,
            Err(error) => self.handle_failure(error).await,
        };
    }

    pub async fn handle_callback_query(&mut self, callback_query: CallbackQuery) {
        let handled = self
            .state
            .handle_callback_query(self.context.clone(), callback_query)
            .await;
        self.state = match handled {
            Ok(state) => state,
            Err(error) => self.handle_failure(error).await,
        };
    }

    pub async fn handle_inline_query(&mut self, inline_query: InlineQuery) {
        let handled = self
            .state
            .handle_inline_query(self.context.clone(), inline_query)
            .await;
        self.state = match handled {
            Ok(state) => state,
            Err(error) => self.handle_failure(error).await,
        };
    }

    async fn handle_failure(&mut self, error: Box<dyn Error + Send + Sync>) -> Box<dyn State> {
        log::error!("Error handling message: {}", error);
        let _ = self
            .context
            .bot
            .send_message(self.context.chat_id, "Something went wrong...")
            .await;
        self.state.clone()
    }
}

#[derive(Clone, Debug)]
pub struct Context {
    pub bot: teloxide::Bot,
    pub chat_id: ChatId,
}

impl Context {
    pub fn new(bot: teloxide::Bot, chat_id: ChatId) -> Self {
        Self { bot, chat_id }
    }
}

type StateOutput = Result<Box<dyn State>, Box<dyn Error + Send + Sync>>;

#[async_trait]
pub trait State: Send + Sync + Debug {
    async fn on_enter(&self) {}

    async fn handle_message(&mut self, _: &Context, _: Message) -> StateOutput {
        Ok(self.clone_state())
    }

    async fn handle_callback_query(&self, _: Context, _: CallbackQuery) -> StateOutput {
        Ok(self.clone_state())
    }

    async fn handle_inline_query(&self, _: Context, _: InlineQuery) -> StateOutput {
        Ok(self.clone_state())
    }

    fn clone_state(&self) -> Box<dyn State>;
}

impl Clone for Box<dyn State> {
    fn clone(&self) -> Box<dyn State> {
        self.clone_state()
    }
}
