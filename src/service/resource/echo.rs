use serde::Serialize;

use crate::error::PipelineResult;

#[derive(Serialize)]
pub struct EchoResponseBody {
	message: String,
}

pub fn get(_: ()) -> PipelineResult<EchoResponseBody> {
	PipelineResult::Ok(EchoResponseBody { message: String::from("mug-bancha says hello!") })
}
