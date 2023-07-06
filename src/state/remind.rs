use async_trait::async_trait;
use chrono::{DateTime, FixedOffset, Utc};
use teloxide::{
    payloads::SendMessageSetters,
    requests::Requester,
    types::{ChatId, Message},
};

use crate::{
    common::{
        config::TIMINGS,
        translation::{self, Translation},
    },
    keyboard,
    prisma::{self, user::next_remind_at},
    state::idle,
    storage::LiteUser,
};

use super::{
    error::{StateError, StateResult},
    State,
};

#[derive(Debug, Clone)]
pub struct Remind {
    word: prisma::word::Data,
    user: LiteUser,
}

impl Remind {
    pub fn new(word: prisma::word::Data, user: LiteUser) -> Remind {
        Remind { word, user }
    }
}

#[async_trait]
impl State for Remind {
    async fn on_enter(&self, ctx: &super::Context, _: Option<Box<dyn State>>) -> StateResult<()> {
        ctx.bot
            .send_message(
                ChatId(self.word.chat_id),
                format!("Write translation for the word `{}`", self.word.word),
            )
            .parse_mode(teloxide::types::ParseMode::MarkdownV2)
            .await?;
        Ok(())
    }

    async fn handle_message(
        &self,
        ctx: &super::Context,
        msg: Message,
    ) -> StateResult<Box<dyn State>> {
        if let Some(text) = msg.text() {
            let translation_request = text.to_owned();
            let translation = Translation::new(&self.word.translate);
            if translation.check(&translation_request) {
                let level = self.word.remember_level + 1;

                if level >= TIMINGS.len() as i32 {
                    ctx.db.remove_word_by_id(self.word.id.clone()).await?;
                    ctx.bot
                        .send_message(msg.chat.id, "🎉 Success! You have remembered the word! 🎊")
                        .await?;
                    return Ok(Box::new(idle::Idle::new()));
                }

                let next_remind_at = calc_next_remind(level)?;
                ctx.db
                    .update_word_remind(self.word.id.clone(), next_remind_at, level)
                    .await?;
                let answer = format!(
                    "🎉 Correct\\! The translation is {}",
                    translation.to_formatted_string()
                );
                ctx.bot
                    .send_message(msg.chat.id, answer)
                    .parse_mode(teloxide::types::ParseMode::MarkdownV2)
                    .await?;
                Ok(Box::new(idle::Idle::new()))
            } else {
                let answer = format!(
                    "😔 Wrong\\! The translation is {}",
                    translation.to_formatted_string()
                );
                ctx.bot
                    .send_message(msg.chat.id, answer)
                    .parse_mode(teloxide::types::ParseMode::MarkdownV2)
                    .await?;
                Ok(Box::new(idle::Idle::new()))
            }
        } else {
            log::error!("Unexpected message without text: {:?}", msg);
            ctx.bot
                .send_message(msg.chat.id, "Unexpected message")
                .await?;
            Ok(Box::new(idle::Idle::new()))
        }
    }

    fn clone_state(&self) -> Box<dyn State> {
        Box::new(self.clone())
    }
}

fn calc_next_remind(level: i32) -> StateResult<DateTime<FixedOffset>> {
    let now = Utc::now();
    let level_timing = TIMINGS
        .get(&level)
        .ok_or(StateError::IncorrectWordLevel(level))?;
    let next_remind = now + chrono::Duration::seconds(level_timing.to_owned());
    let next_remind = next_remind.with_timezone(&FixedOffset::east_opt(0).unwrap());
    Ok(next_remind)
}
