use bytes::Bytes;
use http::{Method, Request, Response, StatusCode};
use hyper::Body;
use serde::{de::DeserializeOwned, Serialize};

use crate::{
	error::{PipelineError, PipelineResult},
	service::{
		resource::{echo, mugs},
		Message,
	},
};

/**
 * This is unstable in Rust for now
type ResponseFuture = impl Future<Item = Response<Body>, Error = hyper::Error>;
 **/
type ResponseFuture = Result<Response<Body>, hyper::Error>;

pub async fn route_request(request: Request<Body>) -> ResponseFuture {
	let (parts, body) = request.into_parts();
	match (parts.uri.path(), parts.method) {
		("/1/echo", Method::GET) => process_request(body, &echo::get).await,
		("/1/mugs", Method::GET) => process_request(body, &mugs::get).await,
		("/1/mugs", Method::PUT) => process_request(body, &mugs::put).await,
		("/1/mugs", Method::PATCH) => process_request(body, &mugs::patch).await,
		("/1/mugs", Method::DELETE) => process_request(body, &mugs::delete).await,
		_ => Ok(error_response(
			StatusCode::NOT_FOUND,
			Message::from("Not found"),
		)),
	}
}

pub async fn process_request<I, O, Handler>(body: Body, handler: &'static Handler) -> ResponseFuture
where
	I: DeserializeOwned,
	O: Serialize,
	Handler: Fn(I) -> PipelineResult<O> + Sync,
{
	let response_future = json_bytes(body).await.map(move |chunk| {
		let result = serde_json::from_slice::<I>(&chunk)
			.map_err(|e| PipelineError::from(e))
			.and_then(|input| handler(input))
			.and_then(|output| serde_json::to_string(&output).map_err(|e| PipelineError::from(e)))
			.and_then(|s| {
				Ok(Response::builder()
					.status(StatusCode::OK)
					.body(Body::from(s))
					.unwrap())
			});
		result.unwrap_or_else(|e| error_response(e.http_status(), Message::from(e.to_string())))
	});
	response_future
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
