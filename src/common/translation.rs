use std::{
    ops::{Deref, DerefMut},
    vec,
};

static SEPARATOR: &str = ", ";

#[derive(Debug, Clone, PartialEq, Hash)]
pub struct Translation(vec::Vec<String>);

impl Deref for Translation {
    type Target = vec::Vec<String>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for Translation {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl Translation {
    pub fn new(text: impl Into<String>) -> Self {
        Self(convert_to_vec(text))
    }

    pub fn to_string(&self) -> String {
        self.0.join(SEPARATOR)
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    pub fn to_formatted_string(&self) -> String {
        self.0
            .iter()
            .map(|s| format!("`{}`", s))
            .collect::<Vec<String>>()
            .join(", ")
    }
}

impl From<&Translation> for String {
    fn from(translation: &Translation) -> Self {
        translation.0.join(SEPARATOR)
    }
}

impl From<Translation> for vec::Vec<String> {
    fn from(translation: Translation) -> Self {
        translation.0
    }
}

impl From<vec::Vec<String>> for Translation {
    fn from(vec: vec::Vec<String>) -> Self {
        Self(
            vec.iter()
                .map(|s| s.trim().to_string())
                .filter(|s| s.is_empty() == false)
                .collect(),
        )
    }
}

impl From<&str> for Translation {
    fn from(text: &str) -> Self {
        Self(convert_to_vec(text))
    }
}

impl From<String> for Translation {
    fn from(text: String) -> Self {
        Self(convert_to_vec(text))
    }
}

impl From<Translation> for String {
    fn from(translation: Translation) -> Self {
        translation.0.join(SEPARATOR)
    }
}

fn convert_to_vec<T>(text: T) -> vec::Vec<String>
where
    T: Into<String>,
{
    text.into()
        .split(SEPARATOR)
        .map(|s| s.trim().to_string())
        .filter(|s| s.is_empty() == false)
        .collect()
}
