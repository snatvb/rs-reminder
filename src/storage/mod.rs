pub mod error;

use std::ops::{Deref, DerefMut};

use chrono::{FixedOffset, Utc};

use crate::{
    common::config::TIMINGS,
    prisma::{self, word},
};

use self::error::{StorageError, StorageResult};

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
    ) -> StorageResult<word::Data> {
        if self.has_word(chat_id, word).await? {
            return Err(StorageError::WordAlreadyExists);
        }

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

    pub async fn has_word(&self, chat_id: i64, word: &str) -> StorageResult<bool> {
        let has_word = self
            .word()
            .find_first(vec![
                word::word::equals(word.to_owned()),
                word::chat_id::equals(chat_id),
            ])
            .exec()
            .await?
            .is_some();
        Ok(has_word)
    }
}
/* #endregion */
