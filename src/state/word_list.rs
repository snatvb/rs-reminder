use async_trait::async_trait;
use teloxide::{
    payloads::SendMessageSetters,
    requests::{Request, Requester},
    types::ParseMode,
};

use crate::keyboard;

use super::{error::StateResult, State};

#[derive(Clone, Debug, Default)]
pub struct WordList {
    offset: i64,
}

pub static WORDS_PER_PAGE: i64 = 20;

impl WordList {
    pub fn new(offset: i64) -> WordList {
        WordList { offset }
    }
}

impl WordList {
    async fn get_words(&self, ctx: &super::Context) -> StateResult<String> {
        let words = ctx
            .db
            .get_words(ctx.chat_id.0, self.offset, WORDS_PER_PAGE)
            .await?;
        let mut text = String::new();
        for word in words {
            text.push_str(&format!("{} \\- `{}`\n", word.word, word.translate));
        }
        Ok(text)
    }
}

#[async_trait]
impl State for WordList {
    async fn on_enter(&self, ctx: &super::Context, _: Option<Box<dyn State>>) -> StateResult<()> {
        log::info!("Entered WordList state");

        let words = self.get_words(ctx).await?;
        ctx.bot
            .send_message(ctx.chat_id, words)
            .parse_mode(ParseMode::MarkdownV2)
            .reply_markup(keyboard::make(&vec![
                vec![
                    Some(keyboard::Button::PrevPage),
                    Some(keyboard::Button::NextPage),
                ],
                vec![Some(keyboard::Button::Cancel)],
            ]))
            .await?;

        Ok(())
    }

    fn clone_state(&self) -> Box<dyn State> {
        Box::new(self.clone())
    }
}
