use futures::future::Future;
use hyper::Method;

use crate::route;
use crate::route::{Endpoint, Router};
use resource::echo;

mod resource;

pub fn run() {
	let addr = ([127, 0, 0, 1], 3000).into();

	let service = || {
		hyper::service::service_fn(|request| {
			Router::with_routes(vec![
				route!(Endpoint("/", &Method::GET),
				       &echo::get,
				       (),
				       echo::EchoResponseBody,
				       "",
				       r#"{"message":"INCREDIBLE"}"#),
			]).dispatch(request)
		})
	};

	let server = hyper::Server::bind(&addr)
	                            .serve(service)
	                            .map_err(|e| eprintln!("hyper error: {}", e));

	hyper::rt::run(server);
}
