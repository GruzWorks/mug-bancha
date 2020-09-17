use bytes::Bytes;
use http::{Response, StatusCode};
use hyper::Body;
use serde::{de::DeserializeOwned, Serialize};

use crate::{
	error::{PipelineError, PipelineResult},
	message::Message,
	service::storage::Storage,
};

pub type ResponseResult = hyper::Result<Response<Body>>;

#[macro_export]
macro_rules! routes {
	(
		$($path:literal => {
			$($method:ident: $handler:expr),* $(,)?
		}),* $(,)?
	) => {
		async fn routes_fn(
			request: http::Request<hyper::Body>,
			storage: Box<dyn Storage>
		) -> $crate::route::ResponseResult {
			let (parts, body) = request.into_parts();
			match parts.uri.path() {
				$( $path => match parts.method {
					$( http::Method::$method => $crate::route::process_request(body, &$handler, storage).await, )*
					_ => Ok($crate::route::error_response(
						http::StatusCode::METHOD_NOT_ALLOWED,
						$crate::message::Message::from("Method not allowed"),
					))
				}, )*
				_ => Ok($crate::route::error_response(
					http::StatusCode::NOT_FOUND,
					$crate::message::Message::from("Not found"),
				))
			}
		}
	};
}

pub async fn process_request<I, O, F, Handler>(
	body: Body,
	handler: &'static Handler,
	storage: Box<dyn Storage>,
) -> ResponseResult
where
	I: DeserializeOwned,
	O: Serialize,
	F: std::future::Future<Output = PipelineResult<O>>,
	Handler: Fn(I, Box<dyn Storage>) -> F,
{
	match json_bytes(body).await {
		Ok(bytes) => {
			let deserialized =
				serde_json::from_slice::<I>(&bytes).map_err(|e| PipelineError::from(e));

			let handled = match deserialized {
				Ok(input) => handler(input, storage).await,
				Err(e) => Err(e),
			};

			let response = handled
				.and_then(|output| {
					serde_json::to_string(&output).map_err(|e| PipelineError::from(e))
				})
				.map(|s| {
					Response::builder()
						.status(StatusCode::OK)
						.body(Body::from(s))
						.unwrap()
				})
				.unwrap_or_else(|e| error_response(e.http_status(), Message::from(e.to_string())));

			Ok(response)
		}
		Err(e) => Err(e),
	}

	/*
	let response_future = json_bytes(body).await.map(move |chunk| {
		let result = serde_json::from_slice::<I>(&chunk)
			.map_err(|e| PipelineError::from(e))
			.and_then(|input| handler(input, storage))
			.and_then(|output| serde_json::to_string(&output).map_err(|e| PipelineError::from(e)))
			.and_then(|s| {
				Ok(Response::builder()
					.status(StatusCode::OK)
					.body(Body::from(s))
					.unwrap())
			});
		result.unwrap_or_else(|e| error_response(e.http_status(), Message::from(e.to_string())))
	});
	response_future*/
}

pub fn error_response(status: StatusCode, msg: Message) -> Response<Body> {
	Response::builder()
		.status(status)
		.body(Body::from(serde_json::to_string(&msg).unwrap()))
		.unwrap()
}

pub async fn json_bytes(body: Body) -> Result<Bytes, hyper::Error> {
	let mut bytes = hyper::body::to_bytes(body).await?;
	if bytes.is_empty() {
		bytes = Bytes::from_static(b"null");
	}
	Ok(bytes)
}
