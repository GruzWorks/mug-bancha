use crate::{error::PipelineResult, service::Message};

pub fn get(_: ()) -> PipelineResult<Message> {
	PipelineResult::Ok(Message::new("mug-bancha says hello!"))
}
