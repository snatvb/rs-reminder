use std::ops::{Deref, DerefMut};

use chrono::{FixedOffset, Utc};
use prisma_client_rust::QueryError;

use crate::{
    common::config::TIMINGS,
    prisma::{self, word},
};

#[derive(Debug)]
pub struct Storage(prisma::PrismaClient);

impl Deref for Storage {
    type Target = prisma::PrismaClient;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for Storage {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl Storage {
    pub fn new(prisma_client: prisma::PrismaClient) -> Self {
        Self(prisma_client)
    }
}

/* #region Word model */
impl Storage {
    pub async fn new_word(
        &self,
        chat_id: i64,
        word: &str,
        translation: &str,
    ) -> Result<word::Data, QueryError> {
        let now = Utc::now();
        let first_remind = now + chrono::Duration::hours(TIMINGS.get(&0i32).unwrap().to_owned());
        let first_remind = first_remind.with_timezone(&FixedOffset::east_opt(0).unwrap());
        let word = self
            .word()
            .create(
                chat_id,
                word.to_owned(),
                translation.to_owned(),
                first_remind,
                vec![],
            )
            .exec()
            .await?;
        Ok(word)
    }
}
/* #endregion */
