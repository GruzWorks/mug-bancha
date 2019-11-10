use std::collections::HashMap;

use bytes::Bytes;
use futures::future;
use futures::future::Future;
use hyper::{Body, Method, Request, Response};
use hyper::rt::Stream;
use http::StatusCode;
use serde::{Serialize, de::DeserializeOwned};
use serde_json::json;

use crate::error::{PipelineResult, PipelineError};

/**
 * This is unstable in Rust for now
type ResponseFuture = impl Future<Item = Response<Body>, Error = hyper::Error>;
 **/
type BoxOfDreams = Box<dyn Future<Item = Response<Body>, Error = hyper::Error> + Send>;

#[derive(PartialEq, Eq, Hash)]
struct Endpoint<'a>(&'a str, &'a Method);

struct Resolution {
	adapter: Box<dyn Fn(Body) -> BoxOfDreams>,
	sample_request_body: &'static str,
	sample_response_body: &'static str,
}

macro_rules! route {
	( $endpoint:expr,
	  $handler:expr,
	  $req_body_type:ty,
	  $resp_body_type:ty,
	  $sample_req_body:expr,
	  $sample_resp_body:expr ) => (
		( $endpoint, Resolution {
			adapter: Box::new(|body| -> BoxOfDreams {
				process_request::<$req_body_type, $resp_body_type>(body, $handler)
			}),
			sample_request_body: $sample_req_body,
			sample_response_body: $sample_resp_body } )
	)
}

fn process_request<I, O>(body: Body, handler: &'static (dyn Fn(I) -> PipelineResult<O> + Sync))
		-> BoxOfDreams
		where I: DeserializeOwned, O: Serialize {
	let response_future = body.concat2()
		.map(|chunk| {
			let mut bytes = chunk.into_bytes();
			if bytes.is_empty() {
				bytes = Bytes::from_static(b"null");
			}
			bytes
		})
		.map(move |chunk| {
			let result = serde_json::from_slice::<I>(&chunk)
				.map_err(|e| PipelineError::from(e))
				.and_then(|input| handler(input))
				.and_then(|output|
					serde_json::to_string(&output)
					            .map_err(|e| PipelineError::from(e)))
				.and_then(|s| Ok(Response::builder()
				                           .status(StatusCode::OK)
				                           .body(Body::from(s))
				                           .unwrap()));
			result.unwrap_or_else(|e| error_response(
					e.http_status(),
					json!({ "message": e.to_string() })))
		});
	Box::new(response_future)
}

fn error_response(status: StatusCode, payload: serde_json::value::Value) -> Response<Body> {
	Response::builder()
	          .status(status)
	          .body(Body::from(payload.to_string()))
	          .unwrap()
}




#[derive(Serialize)]
struct EchoResponseBody {
	message: String,
}

fn echo(_: ()) -> PipelineResult<EchoResponseBody> {
	PipelineResult::Ok(EchoResponseBody { message: String::from("INCREDIBLE") })
}




struct Router<'a> {
	routes: HashMap<Endpoint<'a>, Resolution>,
}

impl<'a> Router<'a> {
	fn init() -> Self {
		Self::with_routes(vec![
			route!(Endpoint("/", &Method::GET),
			       &echo,
			       (),
			       EchoResponseBody,
			       "",
			       r#"{"message":"INCREDIBLE"}"#),
		])
	}

	fn with_routes(r: Vec<(Endpoint<'a>, Resolution)>) -> Self {
		Router { routes: r.into_iter().collect() }
	}

	fn dispatch_request(&self, request: Request<Body>) -> BoxOfDreams {
		let (parts, body) = request.into_parts();
		match self.routes.get(&Endpoint(parts.uri.path(), &parts.method)) {
			Some(resolution) => (resolution.adapter)(body),
			None => Box::new(future::ok(error_response(
				StatusCode::NOT_FOUND,
				json!({ "message": "Not found" })))),
		}
	}
}
/*
		Router::with_routes(vec![
			route!(Endpoint("/", Method::GET),
			       handler,
			       InputDataType,
			       OutputDataType,
			       "",
			       "INCREDIBLE")
		])

*/

pub fn dispatch(request: Request<Body>) -> BoxOfDreams {
	Router::init().dispatch_request(request)
}