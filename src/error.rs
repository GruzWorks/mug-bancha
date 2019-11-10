use std::error::Error;
use std::fmt;

use http::StatusCode;
use serde_json;

pub type PipelineResult<T> = Result<T, PipelineError>;

// TODO include error source information
#[derive(Debug)]
pub enum PipelineError {
	InvalidPayload,
	CannotFulfil,
	InternalIoError,
}

impl PipelineError {
	pub fn http_status(&self) -> StatusCode {
		match self {
			Self::InvalidPayload => StatusCode::BAD_REQUEST,
			Self::CannotFulfil => StatusCode::INTERNAL_SERVER_ERROR,
			Self::InternalIoError => StatusCode::INTERNAL_SERVER_ERROR,
		}
	}
}

impl Error for PipelineError {
	fn source(&self) -> Option<&(dyn Error + 'static)> {
		None
	}
}

impl fmt::Display for PipelineError {
	fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
		match &self {
			Self::InvalidPayload => write!(fmt, "Invalid request payload"),
			Self::CannotFulfil => write!(fmt, "Cannot fulfil request"),
			Self::InternalIoError => write!(fmt, "Internal IO error"),
		}
	}
}

impl From<serde_json::Error> for PipelineError {
	fn from(e: serde_json::Error) -> PipelineError {
		match e.classify() {
			serde_json::error::Category::Io => PipelineError::InternalIoError,
			_ => PipelineError::InvalidPayload,
		}
	}
}
