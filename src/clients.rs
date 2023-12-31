use std::{collections::HashMap, sync::Arc};

use rand::seq::SliceRandom;
use teloxide::{
    requests::ResponseResult,
    types::{CallbackQuery, ChatId, Message},
};

use crate::{
    common::AsyncMutex,
    state::{self, events::Event},
    storage::{users_with_words, Storage},
};

#[derive(Debug, Clone)]
pub struct Client {
    pub fsm: Arc<state::FSM>,
    pub chat_id: ChatId,
}

#[derive(Debug, Clone)]
pub struct Clients {
    clients: Arc<AsyncMutex<HashMap<ChatId, Client>>>,
    bot: Arc<teloxide::Bot>,
    db: Arc<Storage>,
}

impl Clients {
    pub fn new(bot: teloxide::Bot, db: Storage) -> Clients {
        Clients {
            clients: Arc::new(AsyncMutex::new(HashMap::new())),
            bot: Arc::new(bot),
            db: Arc::new(db),
        }
    }

    pub async fn get_or_insert(&self, chat_id: ChatId) -> Client {
        let bot = (*self.bot).clone();
        log::debug!("Lock clients");
        let mut clients = self.clients.lock().await;
        log::debug!("Locked clients");

        if let Some(client) = clients.get(&chat_id) {
            // Client already exists, return a clone of it wrapped in Arc<Mutex>
            return client.clone();
        }

        // // Client doesn't exist, create a new one and insert it into the HashMap
        let client = {
            let context = state::Context::new(bot.clone(), chat_id, self.db.clone());
            let fsm = state::FSM::new(Box::new(state::idle::Idle::new()), context);
            let new_client = Client {
                fsm: Arc::from(fsm),
                chat_id,
            };
            clients.insert(chat_id, new_client.clone());
            new_client
        };
        if let Err(err) = client.fsm.init().await {
            log::error!("Error initializing FSM: {}", err);
        }

        client
    }

    pub async fn handle_message(&self, msg: Message) -> ResponseResult<()> {
        let id = msg.chat.id.to_owned();
        let client = self.get_or_insert(id).await;
        client.fsm.handle_message(msg).await;

        log::debug!("Message handled");
        Ok(())
    }

    pub async fn handle_callback_query(&self, query: CallbackQuery) -> ResponseResult<()> {
        let chat_id = query.message.as_ref().map(|msg| msg.chat.id);
        if let Some(chat_id) = chat_id {
            log::info!("Handling callback query for chat_id {}", chat_id);
            let client = self.get_or_insert(chat_id).await;
            client.fsm.handle_callback_query(query).await;
        }
        log::debug!("Callback query handled");
        Ok(())
    }

    pub async fn handle_event(&self, event: Event) {
        match event {
            Event::Remind => self.remind().await,
            _ => log::warn!("Unknown event: {:?}", event),
        }
    }

    async fn remind(&self) {
        log::debug!("Reminding");
        let users = self.db.find_to_remind().await;
        if let Err(err) = users {
            log::error!("Error getting users to remind: {}", err);
            return;
        }
        let users = users.unwrap();

        let (no_words_users, users_with_words): (
            Vec<users_with_words::Data>,
            Vec<users_with_words::Data>,
        ) = users.iter().cloned().partition(|u| u.words.is_empty());

        log::debug!("Got users to remind: {:?}", users_with_words.len());
        for user in users_with_words {
            let word_opt = user
                .words
                .choose(&mut rand::thread_rng())
                .map(|w| w.clone());
            if let Some(word) = word_opt {
                let chat_id = word.chat_id;
                let client = self.get_or_insert(ChatId(chat_id)).await;
                client
                    .fsm
                    .handle_event(Event::RemindWordToUser(word, user.into()))
                    .await;
            }
        }

        log::debug!("Got users without words: {}", no_words_users.len());
        for user in no_words_users {
            let result = self
                .db
                .update_next_remind_user(user.chat_id, user.remind_every.into())
                .await;
            if let Err(err) = result {
                log::error!("Error updating next remind: {}", err);
            }
        }
    }
}
