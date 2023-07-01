use std::collections::HashMap;

use teloxide::types::ChatId;

use crate::state;

#[derive(Clone, Debug)]
pub struct Client {
    pub fsm: state::FSM,
    pub chat_id: ChatId,
}

pub struct Clients {
    clients: HashMap<ChatId, Client>,
}

impl Clients {
    pub fn new() -> Clients {
        Clients {
            clients: HashMap::new(),
        }
    }

    pub fn get_or_insert(&mut self, bot: teloxide::Bot, chat_id: ChatId) -> &mut Client {
        self.clients.entry(chat_id).or_insert_with(|| {
            let context = state::Context::new(bot, chat_id);
            let fsm = state::FSM::new(Box::new(state::idle::Idle::new()), context);
            Client { fsm, chat_id }
        })
    }
}
