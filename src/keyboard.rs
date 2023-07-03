use teloxide::types::{InlineKeyboardButton, InlineKeyboardMarkup};

use crate::key_value_enum;

key_value_enum! {
    #[derive(Debug, Clone, Copy, Hash, PartialEq)]
    pub enum Button {
        AddWord { text: "Add word", key: "add_word" },
        RemoveWord { text: "Remove word", key: "remove_word" },
        ListWords { text: "List words", key: "list_words" },
        Cancel { text: "Cancel", key: "cancel" },
        PrevPage { text: "←", key: "prev_page" },
        NextPage { text: "→", key: "next_page" },
    }
}

impl Button {
    pub fn to_inline_button(&self) -> InlineKeyboardButton {
        InlineKeyboardButton::callback(self.text(), self.key())
    }

    pub fn to_keyboard(&self) -> InlineKeyboardMarkup {
        InlineKeyboardMarkup::new(vec![vec![self.to_inline_button()]])
    }
}

pub fn make(buttons: &[Vec<Option<Button>>]) -> InlineKeyboardMarkup {
    let keyboard: Vec<Vec<InlineKeyboardButton>> = buttons
        .iter()
        .filter_map(|row| {
            let result: Vec<InlineKeyboardButton> = row
                .iter()
                .flat_map(|button| button.map(|button| button.to_inline_button()))
                .collect();
            if result.is_empty() {
                None
            } else {
                Some(result)
            }
        })
        .collect();
    InlineKeyboardMarkup::new(keyboard)
}

pub fn words_actions() -> InlineKeyboardMarkup {
    let keyboard: Vec<Vec<InlineKeyboardButton>> = vec![
        vec![
            Button::AddWord.to_inline_button(),
            Button::RemoveWord.to_inline_button(),
        ],
        vec![Button::ListWords.to_inline_button()],
    ];

    InlineKeyboardMarkup::new(keyboard)
}
