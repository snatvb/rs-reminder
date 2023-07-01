pub mod add_translation;
pub mod add_word;
pub mod idle;

use async_trait::async_trait;
use std::fmt::Debug;
use tokio::sync::MutexGuard;

use teloxide::{
    requests::{Requester, ResponseResult},
    types::{CallbackQuery, ChatId, InlineQuery, Message},
    RequestError,
};

use crate::common::AsyncMutex;

#[derive(Debug)]
pub struct FSM {
    pub state: AsyncMutex<Box<dyn State>>,
    pub context: Context,
}

impl FSM {
    pub fn new(state: Box<dyn State>, context: Context) -> Self {
        Self {
            state: AsyncMutex::new(state),
            context,
        }
    }

    // pub fn on_enter(&self, from: Option<Box<dyn State>>) -> Pin<Box<dyn Future<Output = ()>>> {
    //     let state = self.state.lock().unwrap();
    //     let future = state.on_enter(&self.context, from);
    //     Box::pin(async move {
    //         future.await;
    //     })
    // }

    pub async fn init(&self) -> ResponseResult<()> {
        log::info!("Init FSM");
        log::debug!("Lock state");
        let state = &self.state.lock().await;
        log::debug!("Locked state");
        state.on_enter(&self.context, None).await?;
        log::info!("Init FSM done");
        Ok(())
    }

    pub async fn handle_message(&self, msg: Message) {
        log::debug!("Handling message: {:?}", msg.text());
        log::debug!("Lock state");
        let current_state = self.state.lock().await;
        log::debug!("Locked state");
        log::debug!("Current state: {:?}", current_state.name());
        let new_state = current_state.handle_message(&self.context, msg).await;
        self.handle_translate(new_state, current_state).await;
    }

    async fn translate(
        &self,
        mut current_state: MutexGuard<'_, Box<dyn State>>,
        new_state: Box<dyn State>,
    ) {
        if current_state.name() != new_state.name() {
            let old_state = current_state.clone_state();
            let handled = new_state.on_enter(&self.context, Some(old_state)).await;
            if let Err(error) = handled {
                self.handle_failure(error).await;
            }
        }

        log::info!(
            "Transitioned from {} to {}",
            current_state.name(),
            new_state.name()
        );
        *current_state = new_state;
    }

    pub async fn handle_callback_query(&self, callback_query: CallbackQuery) {
        let current_state = self.state.lock().await;
        let new_state = current_state
            .handle_callback_query(&self.context, callback_query)
            .await;
        self.handle_translate(new_state, current_state).await;
    }

    async fn handle_translate(
        &self,
        new_state: Result<Box<dyn State>, RequestError>,
        current_state: MutexGuard<'_, Box<dyn State>>,
    ) {
        if let Err(error) = new_state {
            self.handle_failure(error).await;
        } else {
            self.translate(current_state, new_state.unwrap()).await;
        }
    }

    // pub async fn handle_inline_query(&mut self, inline_query: InlineQuery) {
    //     let handled = self
    //         .state
    //         .handle_inline_query(self.context.clone(), inline_query)
    //         .await;
    //     self.state = match handled {
    //         Ok(state) => state,
    //         Err(error) => self.handle_failure(error).await,
    //     };
    // }

    async fn handle_failure(&self, error: RequestError) -> Box<dyn State> {
        log::error!("Error handling message: {}", error);
        let _ = self
            .context
            .bot
            .send_message(self.context.chat_id, "Something went wrong...")
            .await;
        self.state.lock().await.clone()
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

#[async_trait]
pub trait State: Send + Sync + Debug {
    fn name(&self) -> &'static str {
        std::any::type_name::<Self>()
    }

    async fn on_enter(&self, _: &Context, _: Option<Box<dyn State>>) -> ResponseResult<()> {
        Ok(())
    }

    async fn handle_message(&self, _: &Context, _: Message) -> ResponseResult<Box<dyn State>> {
        Ok(self.clone_state())
    }

    async fn handle_callback_query(
        &self,
        _: &Context,
        _: CallbackQuery,
    ) -> ResponseResult<Box<dyn State>> {
        Ok(self.clone_state())
    }

    async fn handle_inline_query(
        &self,
        _: &Context,
        _: InlineQuery,
    ) -> ResponseResult<Box<dyn State>> {
        Ok(self.clone_state())
    }

    fn clone_state(&self) -> Box<dyn State>;
}

impl Clone for Box<dyn State> {
    fn clone(&self) -> Box<dyn State> {
        self.clone_state()
    }
}
