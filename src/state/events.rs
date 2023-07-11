use teloxide::types::{CallbackQuery, Message};

use crate::{keyboard::Button, prisma, storage::LiteUser};

#[derive(Debug, Clone)]
pub enum Event {
    Message(Message),
    Button(Button, CallbackQuery),
    Remind,
    Timeout,
    RemindWord(prisma::word::Data),
    RemindWordToUser(prisma::word::Data, LiteUser),
}
