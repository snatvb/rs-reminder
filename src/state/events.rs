use teloxide::types::{CallbackQuery, Message};

use crate::prisma;

#[derive(Debug, Clone)]
pub enum Event {
    Message(Message),
    CallbackQuery(CallbackQuery),
    Remind,
    RemindWord(prisma::word::Data),
}
