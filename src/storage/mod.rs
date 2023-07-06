pub mod error;

use std::ops::{Deref, DerefMut};

use chrono::{FixedOffset, Utc};

use crate::{
    common::config::TIMINGS,
    prisma::{self, user, word},
};

use self::error::{StorageError, StorageResult};

pub static MAX_USERS_TO_REMIND: i64 = 200;

user::include!((filters: Vec<word::WhereParam>) => users_with_words {
    words(filters)
});

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

/* #region User model */
impl Storage {
    pub async fn get_user(&self, chat_id: i64) -> StorageResult<Option<user::Data>> {
        let user = self
            .user()
            .find_first(vec![user::chat_id::equals(chat_id)])
            .exec()
            .await?;
        Ok(user)
    }

    pub async fn new_user(&self, chat_id: i64) -> StorageResult<user::Data> {
        let user = self.user().create(chat_id, vec![]).exec().await?;
        Ok(user)
    }

    pub async fn ensure_user(&self, chat_id: i64) -> StorageResult<user::Data> {
        let user = self.get_user(chat_id).await?;
        if let Some(user) = user {
            Ok(user)
        } else {
            self.new_user(chat_id).await
        }
    }

    pub async fn update_next_remind(&self, id: i64, remind_every: i64) -> StorageResult<()> {
        let now = Utc::now();
        let next_remind_at = now + chrono::Duration::seconds(remind_every as i64);
        let next_remind_at = next_remind_at.with_timezone(&FixedOffset::east_opt(0).unwrap());

        self.user()
            .update(
                user::chat_id::equals(id),
                vec![user::next_remind_at::set(next_remind_at)],
            )
            .exec()
            .await?;
        Ok(())
    }
}
/* #endregion */

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

        let user = self.ensure_user(chat_id).await?;

        let now = Utc::now();
        let first_remind = now + chrono::Duration::seconds(TIMINGS.get(&0i32).unwrap().to_owned());
        let first_remind = first_remind.with_timezone(&FixedOffset::east_opt(0).unwrap());
        let word = self
            .word()
            .create(
                chat_id,
                word.to_owned(),
                translation.to_owned(),
                first_remind,
                prisma::user::UniqueWhereParam::ChatIdEquals(user.chat_id),
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

    pub async fn get_words(
        &self,
        chat_id: i64,
        skip: i64,
        take: i64,
    ) -> StorageResult<Vec<word::Data>> {
        let words = self
            .word()
            .find_many(vec![word::chat_id::equals(chat_id)])
            .skip(skip)
            .take(take)
            .exec()
            .await?;
        Ok(words)
    }

    pub async fn words_count(&self, chat_id: i64) -> StorageResult<i64> {
        let count = self
            .word()
            .count(vec![word::chat_id::equals(chat_id)])
            .exec()
            .await?;
        Ok(count)
    }

    pub async fn remove_word(&self, chat_id: i64, word: &str) -> StorageResult<()> {
        self.word()
            .delete_many(vec![
                word::word::equals(word.to_owned()),
                word::chat_id::equals(chat_id),
            ])
            .exec()
            .await?;
        Ok(())
    }

    pub async fn find_to_remind(&self) -> StorageResult<Vec<users_with_words::Data>> {
        let now = Utc::now();
        let fixed_now = now.with_timezone(&FixedOffset::east_opt(0).unwrap());
        let word_filters = vec![word::next_remind_at::lte(fixed_now)];
        let users = self
            .user()
            .find_many(vec![user::next_remind_at::lte(fixed_now)])
            .take(MAX_USERS_TO_REMIND)
            .include(users_with_words::include(word_filters))
            .exec()
            .await?;

        log::debug!("Found {} users to remind", users.len());

        Ok(users)
    }
}
/* #endregion */
