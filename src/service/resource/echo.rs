use serde::{Deserialize, Serialize};

use crate::error::PipelineResult;

#[derive(Debug, Deserialize, PartialEq, Serialize)]
pub struct EchoResponseBody {
	pub message: String,
}

pub fn get(_: ()) -> PipelineResult<EchoResponseBody> {
	PipelineResult::Ok(EchoResponseBody {
		message: String::from("mug-bancha says hello!"),
	})
}
