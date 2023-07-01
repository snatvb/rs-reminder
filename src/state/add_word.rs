use super::State;

#[derive(Clone, Debug)]
pub struct AddWord {}

impl AddWord {
    pub fn new() -> AddWord {
        AddWord {}
    }
}

impl State for AddWord {
    fn clone_state(&self) -> Box<dyn State> {
        Box::new(self.clone())
    }
}
