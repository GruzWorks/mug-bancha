use std::sync::Mutex;

use futures::future::Future;
use hyper::Method;

use crate::route;
use crate::route::{Endpoint, Router};
use resource::echo;
use resource::mugs;

mod resource;

pub fn run() {
	unsafe {
		mugs::MUGS.storage = Some(Mutex::new(mugs::Storage::init()));
	}

	let addr = ([127, 0, 0, 1], 3000).into();

	let service = || {
		hyper::service::service_fn(|request| {
			Router::with_routes(vec![
				route!(Endpoint("/1/echo", &Method::GET),
				       &echo::get,
				       (),
				       echo::EchoResponseBody,
				       "",
				       r#"{"message":"INCREDIBLE"}"#),
				route!(Endpoint("/1/mugs", &Method::GET),
				       &mugs::get,
				       (),
				       Vec<mugs::Mug>,
				       "",
				       "[]"),
				route!(Endpoint("/1/mugs", &Method::PUT),
				       &mugs::put,
				       mugs::EphemeralMug,
				       mugs::Mug,
				       "",
				       ""),
				route!(Endpoint("/1/mugs", &Method::PATCH),
				       &mugs::patch,
				       mugs::Mug,
				       mugs::Mug,
				       "",
				       ""),
				route!(Endpoint("/1/mugs", &Method::DELETE),
				       &mugs::delete,
				       mugs::Mug,
				       (),
				       "",
				       ""),
			]).dispatch(request)
		})
	};

	let server = hyper::Server::bind(&addr)
	                            .serve(service)
	                            .map_err(|e| eprintln!("hyper error: {}", e));

	hyper::rt::run(server);
}
