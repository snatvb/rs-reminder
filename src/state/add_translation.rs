use super::State;

#[derive(Clone)]
struct AddTranslation {
    word: String,
}

impl AddTranslation {
    pub fn new(word: &str) -> AddTranslation {
        AddTranslation {
            word: word.to_owned(),
        }
    }
}

impl State for AddTranslation {
    fn clone_state(&self) -> Box<dyn State> {
        Box::new(self.clone())
    }
}
