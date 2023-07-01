pub mod add_translation;
pub mod add_word;
pub mod error;
pub mod idle;

use async_trait::async_trait;
use std::{fmt::Debug, sync::Arc};
use tokio::sync::MutexGuard;

use teloxide::{
    requests::Requester,
    types::{CallbackQuery, ChatId, InlineQuery, Message},
};

use crate::{
    common::AsyncMutex,
    keyboard,
    storage::{error::StorageError, Storage},
};

use self::error::{StateError, StateResult};

#[derive(Debug)]
pub struct FSM {
    pub state: AsyncMutex<Box<dyn State>>,
    pub context: Arc<Context>,
}

impl FSM {
    pub fn new(state: Box<dyn State>, context: Context) -> Self {
        Self {
            state: AsyncMutex::new(state),
            context: Arc::new(context),
        }
    }

    // pub fn on_enter(&self, from: Option<Box<dyn State>>) -> Pin<Box<dyn Future<Output = ()>>> {
    //     let state = self.state.lock().unwrap();
    //     let future = state.on_enter(&self.context, from);
    //     Box::pin(async move {
    //         future.await;
    //     })
    // }

    pub async fn init(&self) -> StateResult<()> {
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
        let current_state = self.state.lock().await;
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
        if self.is_cancel_cmd(&callback_query) {
            self.handle_translate(Ok(Box::new(idle::Idle::new())), current_state)
                .await;
            if let Some(Message { id, chat, .. }) = callback_query.message {
                let _ = self.context.bot.delete_message(chat.id, id).await;
            }
            return;
        };

        let new_state = current_state
            .handle_callback_query(&self.context, callback_query)
            .await;
        self.handle_translate(new_state, current_state).await;
    }

    fn is_cancel_cmd(&self, query: &CallbackQuery) -> bool {
        if let Some(Ok(cmd)) = query
            .data
            .to_owned()
            .map(|text| keyboard::Button::from_key(&text))
        {
            return cmd == keyboard::Button::Cancel;
        }
        return false;
    }

    async fn handle_translate(
        &self,
        new_state: StateResult<Box<dyn State>>,
        current_state: MutexGuard<'_, Box<dyn State>>,
    ) {
        match new_state {
            Ok(state) => self.translate(current_state, state).await,
            Err(error) => {
                self.handle_failure(error).await;
                let idle_state = Box::new(idle::Idle::new());
                if current_state.name() != idle_state.name() {
                    self.translate(current_state, idle_state).await;
                }
            }
        }
    }

    async fn handle_failure(&self, error: StateError) {
        log::error!("Error handling message: {}", error);

        match error {
            StateError::StorageError(error) => self.handle_storage_error(error).await,
            _ => self.answer_smt_went_wrong().await,
        }
    }

    async fn handle_storage_error(&self, error: StorageError) {
        match error {
            StorageError::WordAlreadyExists => {
                let _ = self
                    .context
                    .bot
                    .send_message(self.context.chat_id, "Word already exists")
                    .await;
            }
            _ => self.answer_smt_went_wrong().await,
        }
    }

    async fn answer_smt_went_wrong(&self) {
        let _ = self
            .context
            .bot
            .send_message(self.context.chat_id, "Something went wrong...")
            .await;
    }
}

#[derive(Clone, Debug)]
pub struct Context {
    pub bot: teloxide::Bot,
    pub chat_id: ChatId,
    pub db: Arc<Storage>,
}

impl Context {
    pub fn new(bot: teloxide::Bot, chat_id: ChatId, db: Arc<Storage>) -> Self {
        Self { bot, chat_id, db }
    }
}

#[async_trait]
pub trait State: Send + Sync + Debug {
    fn name(&self) -> &'static str {
        std::any::type_name::<Self>()
    }

    async fn on_enter(&self, _: &Context, _: Option<Box<dyn State>>) -> StateResult<()> {
        Ok(())
    }

    async fn handle_message(&self, _: &Context, _: Message) -> StateResult<Box<dyn State>> {
        Ok(self.clone_state())
    }

    async fn handle_callback_query(
        &self,
        _: &Context,
        _: CallbackQuery,
    ) -> StateResult<Box<dyn State>> {
        Ok(self.clone_state())
    }

    async fn handle_inline_query(
        &self,
        _: &Context,
        _: InlineQuery,
    ) -> StateResult<Box<dyn State>> {
        Ok(self.clone_state())
    }

    fn clone_state(&self) -> Box<dyn State>;
}

impl Clone for Box<dyn State> {
    fn clone(&self) -> Box<dyn State> {
        self.clone_state()
    }
}
