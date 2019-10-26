use std::collections::HashMap;

use futures::future;
use futures::future::{Either, Future};
use hyper::{Body, Chunk, Method, Request, Response};
use hyper::rt::Stream;
use serde::{Serialize, Deserialize, de::DeserializeOwned};
use serde_json::json;

/**
 * This is unstable in Rust for now
type ResponseFuture = impl Future<Item = Response<Body>, Error = hyper::Error>;
 **/
type BoxOfDreams = Box<dyn Future<Item = Response<Body>, Error = hyper::Error> + Send>;

enum RequestResult<T> {
	Ok(T),
	InvalidPayload,
}

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

fn process_request<I, O>(body: Body, handler: &'static (dyn Fn(I) -> RequestResult<O> + Sync))
		-> BoxOfDreams
		where I: DeserializeOwned, O: Serialize {
	let response_future = body.concat2().map(move |chunk| {
		match serde_json::from_slice::<I>(&chunk) {
			Ok(input) => {
				match handler(input) {
					RequestResult::Ok(output) => {
						match serde_json::to_string(&output) {
							Ok(json) => Response::builder()
							                      .status(200)
							                      .body(Body::from(json))
							                      .unwrap(),
							Err(e) => Response::builder()
							                    .status(500)
							                    .body(bodify(json!({ "message": format!("Response serialization failed: {}", e) })))
							                    .unwrap(),
						}
					},
					_ => Response::builder()
					               .status(500)
					               .body(bodify(json!({ "message": "Cannot fulfil request" })))
					               .unwrap(),
				}
			},
			Err(e) => Response::builder()
			                    .status(400)
			                    .body(bodify(json!({ "message": format!("Invalid request payload: {}", e) })))
			                    .unwrap(),
		}
	});
	Box::new(response_future)
}

fn bodify(value: serde_json::value::Value) -> Body {
	Body::from(value.to_string())
}




#[derive(Serialize)]
struct EchoResponseBody {
	message: String,
}

fn echo(_: ()) -> RequestResult<EchoResponseBody> {
	RequestResult::Ok(EchoResponseBody { message: String::from("INCREDIBLE") })
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
/*
		Self::with_routes(vec![
			(Endpoint("/", Method::GET),
			 Resolution { adapter: Box::new(|_request| { Response::new(Body::from("INCREDIBLE")) }),
			              sample_request_body: "",
			              sample_response_body: "INCREDIBLE" })
		])
*/
	}

	fn with_routes(r: Vec<(Endpoint<'a>, Resolution)>) -> Self {
		Router { routes: r.into_iter().collect() }
	}

	fn dispatch_request(&self, request: Request<Body>) -> BoxOfDreams {
		let (parts, body) = request.into_parts();
		match self.routes.get(&Endpoint(parts.uri.path(), &parts.method)) {
			Some(resolution) => (resolution.adapter)(body),
			None => Box::new(future::ok(Response::builder()
			                             .status(404)
			                             .body(bodify(json!({ "message": "Not found" })))
			                             .unwrap())),
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
