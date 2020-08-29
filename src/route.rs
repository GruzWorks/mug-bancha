use bytes::Bytes;
use futures::{future, Future};
use http::{Method, Request, Response, StatusCode};
use hyper::{Body, rt::Stream};
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
type ResponseFuture = Box<dyn Future<Item = Response<Body>, Error = hyper::Error> + Send>;

pub fn route_request(request: Request<Body>) -> ResponseFuture {
	let (parts, body) = request.into_parts();
	match (parts.uri.path(), parts.method) {
		("/1/echo", Method::GET) => process_request(body, &echo::get),
		("/1/mugs", Method::GET) => process_request(body, &mugs::get),
		("/1/mugs", Method::PUT) => process_request(body, &mugs::put),
		("/1/mugs", Method::PATCH) => process_request(body, &mugs::patch),
		("/1/mugs", Method::DELETE) => process_request(body, &mugs::delete),
		_ => Box::new(future::ok(error_response(
			StatusCode::NOT_FOUND,
			Message::from("Not found"),
		))),
	}
}

pub fn process_request<I, O, Handler>(body: Body, handler: &'static Handler) -> ResponseFuture
where
	I: DeserializeOwned,
	O: Serialize,
	Handler: Fn(I) -> PipelineResult<O> + Sync,
{
	let response_future = json_bytes(body).map(move |chunk| {
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
	Box::new(response_future)
}

pub fn error_response(status: StatusCode, msg: Message) -> Response<Body> {
	Response::builder()
		.status(status)
		.body(Body::from(serde_json::to_string(&msg).unwrap()))
		.unwrap()
}

pub fn json_bytes(body: Body) -> impl Future<Item = Bytes, Error = hyper::Error> {
	body.concat2().map(|chunk| {
		let mut bytes = chunk.into_bytes();
		if bytes.is_empty() {
			bytes = Bytes::from_static(b"null");
		}
		bytes
	})
}
