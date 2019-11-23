use std::borrow::Borrow;
use std::error::Error;
use std::fmt;

use http::StatusCode;
use serde_json;

pub type PipelineResult<T> = Result<T, PipelineError>;

// TODO check if storing source Errors is necessary
#[derive(Debug)]
pub enum PipelineError {
	InvalidPayload(Box<dyn Error>),
	InternalIoError(Box<dyn Error>),
	CannotFulfil,
	NotFound(String),
}

impl PipelineError {
	pub fn http_status(&self) -> StatusCode {
		match self {
			Self::InvalidPayload(_) => StatusCode::BAD_REQUEST,
			Self::InternalIoError(_) => StatusCode::INTERNAL_SERVER_ERROR,
			Self::CannotFulfil => StatusCode::INTERNAL_SERVER_ERROR,
			Self::NotFound(_) => StatusCode::NOT_FOUND,
		}
	}
}

impl Error for PipelineError {
	fn source(&self) -> Option<&(dyn Error + 'static)> {
		match self {
			Self::InvalidPayload(source) |
			Self::InternalIoError(source) => Some(source.borrow()),
			_ => None,
		}
	}
}

impl fmt::Display for PipelineError {
	fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
		match &self {
			Self::InvalidPayload(e) => write!(fmt, "Invalid request payload: {}", e.to_string()),
			Self::InternalIoError(e) => write!(fmt, "Internal IO error: {}", e.to_string()),
			Self::CannotFulfil => write!(fmt, "Cannot fulfil request"),
			Self::NotFound(s) => write!(fmt, "Not found: {}", s),
		}
	}
}

impl From<serde_json::Error> for PipelineError {
	fn from(e: serde_json::Error) -> PipelineError {
		match e.classify() {
			serde_json::error::Category::Io => PipelineError::InternalIoError(Box::new(e)),
			_ => PipelineError::InvalidPayload(Box::new(e)),
		}
	}
}
