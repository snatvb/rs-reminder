use async_trait::async_trait;
use teloxide::requests::ResponseResult;

use super::State;

#[derive(Clone, Debug)]
pub struct AddWord {}

impl AddWord {
    pub fn new() -> AddWord {
        AddWord {}
    }
}

#[async_trait]
impl State for AddWord {
    async fn on_enter(&self, _: &super::Context, _: Option<Box<dyn State>>) -> ResponseResult<()> {
        log::info!("Entered AddWord state");

        Ok(())
    }

    fn clone_state(&self) -> Box<dyn State> {
        Box::new(self.clone())
    }
}
