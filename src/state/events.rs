use teloxide::types::{CallbackQuery, Message};

use crate::{prisma, storage::LiteUser};

#[derive(Debug, Clone)]
pub enum Event {
    Message(Message),
    CallbackQuery(CallbackQuery),
    Remind,
    RemindWord(prisma::word::Data),
    RemindWordToUser(prisma::word::Data, LiteUser),
}
