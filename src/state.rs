pub mod add_translation;
pub mod add_word;
pub mod error;
pub mod events;
pub mod idle;
pub mod remind;
pub mod remove_words;
pub mod word_list;

use async_trait::async_trait;
use std::{fmt::Debug, sync::Arc, time};
use tokio::{sync::MutexGuard, task::JoinHandle};

use teloxide::{
    requests::Requester,
    types::{CallbackQuery, ChatId, InlineQuery, Message},
    utils::command::BotCommands,
};

use crate::{
    common::{AsyncMutex, Command},
    keyboard,
    storage::{error::StorageError, Storage},
};

use self::{
    error::{StateError, StateResult},
    events::Event,
};

const DEFAULT_STATE_TIMEOUT: u64 = 60; // seconds

#[derive(Debug, Clone)]
pub struct FSM {
    pub state: Arc<AsyncMutex<Box<dyn State>>>,
    pub context: Arc<Context>,

    timeout: Arc<AsyncMutex<Option<JoinHandle<()>>>>,
}

impl FSM {
    pub fn new(state: Box<dyn State>, context: Context) -> Self {
        Self {
            state: Arc::new(AsyncMutex::new(state)),
            context: Arc::new(context),
            timeout: Arc::new(AsyncMutex::new(None)),
        }
    }

    pub async fn init(&self) -> StateResult<()> {
        log::info!("Init FSM");
        log::debug!("Lock state");
        let state = &self.state.lock().await;
        log::debug!("Locked state");
        state.on_enter(&self.context, None).await?;
        log::info!("Init FSM done");
        Ok(())
    }

    async fn parse_command(&self, msg: &Message) -> Option<Command> {
        if let Some(text) = msg.text() {
            if let Ok(me) = self.context.bot.get_me().await {
                return BotCommands::parse(text, me.username()).ok();
            }
        }
        None
    }

    async fn handle_command(&self, msg: &Message, cmd: Command) -> StateResult<()> {
        match cmd {
            Command::Help => {
                let response = Command::descriptions().to_string();
                self.context.bot.send_message(msg.chat.id, response).await?;
            }

            // hard reset to idle state
            Command::Start => {
                let current_state = self.state.lock().await;
                let idle_state = idle::Idle::new();
                if current_state.name() == idle_state.name() {
                    idle::Idle::send_start_msg(&self.context).await?;
                } else {
                    self.change_state(current_state, Box::new(idle_state)).await;
                }
            }
        }
        Ok(())
    }

    pub async fn handle_message(&self, msg: Message) {
        log::debug!("Handling message: {:?}", msg.text());
        if let Some(cmd) = self.parse_command(&msg).await {
            log::debug!("Command: {:?}", cmd);
            if let Err(error) = self.handle_command(&msg, cmd).await {
                self.handle_failure(error).await;
            }
            return;
        }

        let current_state = self.state.lock().await;
        log::debug!("Current state: {:?}", current_state.name());
        let new_state = current_state.handle_message(&self.context, msg).await;
        self.handle_new_state(new_state, current_state).await;
    }

    async fn change_state(
        &self,
        current_state: MutexGuard<'_, Box<dyn State>>,
        new_state: Box<dyn State>,
    ) {
        let updated_state = current_state.name() != new_state.name();
        if updated_state == false {
            return;
        }

        self.abort_timeout().await;
        let timeout_duration = new_state.timeout();
        self.translate_state(current_state, new_state).await;

        if let Some(timeout_duration) = timeout_duration {
            log::debug!("Setting timeout to {:?}", timeout_duration);
            self.set_timeout(timeout_duration).await;
        }
    }

    async fn translate_state(
        &self,
        mut current_state: MutexGuard<'_, Box<dyn State>>,
        new_state: Box<dyn State>,
    ) {
        let old_state = current_state.clone_state();
        let handled = new_state.on_enter(&self.context, Some(old_state)).await;
        if let Err(error) = handled {
            self.handle_failure(error).await;
        }
        log::info!(
            "Transitioned from {} to {}",
            current_state.name(),
            new_state.name()
        );
        *current_state = new_state;
    }

    async fn set_timeout(&self, timeout_duration: time::Duration) {
        let me = self.clone();
        let mut timeout = self.timeout.lock().await;
        *timeout = Some(tokio::spawn(async move {
            tokio::time::sleep(timeout_duration).await;
            log::info!("Timeout expired");
            let current_state = me.state.lock().await;
            let new_state = current_state.handle_timeout(&me.context).await;
            match new_state {
                Ok(new_state) => me.translate_state(current_state, new_state).await,
                Err(error) => {
                    me.handle_failure(error).await;
                    me.translate_state(current_state, Box::new(idle::Idle::new()))
                        .await;
                }
            }
        }));
    }

    async fn abort_timeout(&self) {
        let mut timeout = self.timeout.lock().await;
        if let Some(timeout) = timeout.as_mut() {
            log::debug!("Cancelling timeout");
            timeout.abort();
        }
        *timeout = None;
    }

    pub async fn handle_callback_query(&self, callback_query: CallbackQuery) {
        let current_state = self.state.lock().await;

        if let Some(Ok(cmd)) = callback_query
            .data
            .to_owned()
            .map(|text| keyboard::Button::from_key(&text))
        {
            match cmd {
                keyboard::Button::Cancel => {
                    self.handle_new_state(Ok(Box::new(idle::Idle::new())), current_state)
                        .await;
                    if let Some(Message { id, chat, .. }) = callback_query.message {
                        let _ = self.context.bot.delete_message(chat.id, id).await;
                    }
                }
                _ => {
                    let new_state = current_state
                        .handle_event(&self.context, events::Event::Button(cmd, callback_query))
                        .await;
                    self.handle_new_state(new_state, current_state).await;
                }
            }
        } else {
            let new_state = current_state
                .handle_callback_query(&self.context, callback_query)
                .await;
            self.handle_new_state(new_state, current_state).await;
        }
    }

    pub async fn handle_event(&self, event: Event) {
        let current_state = self.state.lock().await;
        let new_state = current_state.handle_event(&self.context, event).await;
        self.handle_new_state(new_state, current_state).await;
    }

    async fn handle_new_state(
        &self,
        new_state: StateResult<Box<dyn State>>,
        current_state: MutexGuard<'_, Box<dyn State>>,
    ) {
        match new_state {
            Ok(state) => self.change_state(current_state, state).await,
            Err(error) => {
                self.handle_failure(error).await;
                let idle_state = Box::new(idle::Idle::new());
                if current_state.name() != idle_state.name() {
                    self.change_state(current_state, idle_state).await;
                }
            }
        }
    }

    async fn handle_failure(&self, error: StateError) {
        log::error!("Error handling message: {:?}", error);

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

    fn timeout(&self) -> Option<time::Duration> {
        Some(time::Duration::from_secs(DEFAULT_STATE_TIMEOUT))
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

    async fn handle_event(&self, _: &Context, _: Event) -> StateResult<Box<dyn State>> {
        Ok(self.clone_state())
    }

    async fn handle_timeout(&self, _: &Context) -> StateResult<Box<dyn State>> {
        Ok(Box::new(idle::Idle::new()))
    }

    fn clone_state(&self) -> Box<dyn State>;
}

impl Clone for Box<dyn State> {
    fn clone(&self) -> Box<dyn State> {
        self.clone_state()
    }
}
