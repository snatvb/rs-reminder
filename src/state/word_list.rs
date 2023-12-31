use async_trait::async_trait;
use teloxide::{
    payloads::{EditMessageTextSetters, SendMessageSetters},
    requests::Requester,
    types::{MessageId, ParseMode},
};

use crate::keyboard;

use super::{
    error::{StateError, StateResult},
    events::Event,
    State,
};

pub static WORDS_PER_PAGE: i64 = 5;

#[derive(Clone, Debug, Default)]
pub struct WordList {
    message_id: Option<MessageId>,
    offset: i64,
}

impl WordList {
    pub fn new(message_id: Option<MessageId>, offset: i64) -> WordList {
        WordList { message_id, offset }
    }
}

impl WordList {
    async fn get_words(&self, ctx: &super::Context, total_words: i64) -> StateResult<String> {
        let offset = num::clamp(
            self.offset,
            0,
            std::cmp::max(0, total_words - WORDS_PER_PAGE),
        );
        let words = ctx
            .db
            .get_words(ctx.chat_id.0, offset, WORDS_PER_PAGE)
            .await?;
        let mut text = String::new();
        for word in words {
            text.push_str(&format!(
                "{} \\- `{}` \\(level: *{}*\\)\n",
                word.word, word.translate, word.remember_level
            ));
        }
        Ok(text)
    }

    async fn update_list(&self, ctx: &super::Context) -> StateResult<()> {
        let total_amount = ctx.db.words_count(ctx.chat_id.0).await?;
        let words = self.get_words(ctx, total_amount).await?;
        let prev_button = if self.offset > 0 {
            Some(keyboard::Button::PrevPage)
        } else {
            None
        };
        let next_button = if self.offset + WORDS_PER_PAGE < total_amount {
            Some(keyboard::Button::NextPage)
        } else {
            None
        };

        let keyboard_of_list = keyboard::make(&vec![
            vec![prev_button, next_button],
            vec![Some(keyboard::Button::Cancel)],
        ]);
        if let Some(msg_id) = self.message_id {
            ctx.bot
                .edit_message_text(ctx.chat_id, msg_id, words)
                .parse_mode(ParseMode::MarkdownV2)
                .reply_markup(keyboard_of_list)
                .await?;
        } else {
            ctx.bot
                .send_message(ctx.chat_id, words)
                .parse_mode(ParseMode::MarkdownV2)
                .reply_markup(keyboard_of_list)
                .await?;
        };
        Ok(())
    }

    async fn next_page(&self, ctx: &super::Context) -> StateResult<Box<dyn State>> {
        let new_state = WordList::new(self.message_id, self.offset + WORDS_PER_PAGE);
        new_state.update_list(ctx).await?;
        Ok(Box::new(new_state))
    }

    async fn prev_page(&self, ctx: &super::Context) -> StateResult<Box<dyn State>> {
        let new_state = WordList::new(self.message_id, self.offset - WORDS_PER_PAGE);
        new_state.update_list(ctx).await?;
        Ok(Box::new(new_state))
    }
}

#[async_trait]
impl State for WordList {
    async fn on_enter(&self, ctx: &super::Context, _: Option<Box<dyn State>>) -> StateResult<()> {
        log::info!("Entered WordList state");

        self.update_list(ctx).await
    }

    async fn handle_event(
        &self,
        ctx: &super::Context,
        event: Event,
    ) -> StateResult<Box<dyn State>> {
        if let Event::Button(cmd, _) = event {
            return match cmd {
                keyboard::Button::NextPage => self.next_page(ctx).await,
                keyboard::Button::PrevPage => self.prev_page(ctx).await,
                _ => Err(StateError::UnexpectedCommand(format!(
                    "Unexpected command {} - {}",
                    cmd.key(),
                    cmd.text(),
                ))),
            };
        }

        Ok(self.clone_state())
    }

    fn clone_state(&self) -> Box<dyn State> {
        Box::new(self.clone())
    }
}
