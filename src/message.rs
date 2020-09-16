use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, PartialEq, Serialize)]
pub struct Message {
	pub message: String,
}

impl From<&'_ str> for Message {
	fn from(s: &'_ str) -> Self {
		Message {
			message: s.to_owned(),
		}
	}
}

impl From<String> for Message {
	fn from(s: String) -> Self {
		Message { message: s }
	}
}
