use crate::{error::PipelineResult, service::storage::Storage, service::Message};

pub async fn get(_: (), _: Box<dyn Storage>) -> PipelineResult<Message> {
	PipelineResult::Ok(Message::from("mug-bancha says hello!"))
}
