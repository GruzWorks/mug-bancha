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
				       r#"[{"id":4,"name":"Foo","lat":51.0,"lon":17.0,"address":"14 Bar, Baz 2222, Fooland","num_mugs":2}]"#),
				route!(Endpoint("/1/mugs", &Method::PUT),
				       &mugs::put,
				       mugs::EphemeralMug,
				       mugs::Mug,
				       r#"{"name":"A Field","lat":54.0,"lon":-1.0,"address":"Stockton-on-the-Forest, York YO32 9WB, United Kingdom","num_mugs":1}"#,
				       r#"{"id":-2578664604024157846,"name":"A Field","lat":54.0,"lon":-1.0,"address":"Stockton-on-the-Forest, York YO32 9WB, United Kingdom","num_mugs":1}"#),
				route!(Endpoint("/1/mugs", &Method::PATCH),
				       &mugs::patch,
				       mugs::Mug,
				       mugs::Mug,
				       r#"{"id":4,"name":"Foo","lat":51.12,"lon":17.17,"address":"14 Bar, Baz 2222, Fooland","num_mugs":2}"#,
				       r#"{"id":4,"name":"Foo","lat":51.12,"lon":17.17,"address":"14 Bar, Baz 2222, Fooland","num_mugs":2}"#),
				route!(Endpoint("/1/mugs", &Method::DELETE),
				       &mugs::delete,
				       mugs::Mug,
				       (),
				       r#"{"id":4,"name":"Foo","lat":51.0,"lon":17.0,"address":"14 Bar, Baz 2222, Fooland","num_mugs":2}"#,
				       "null"),
			]).dispatch(request)
		})
	};

	let server = hyper::Server::bind(&addr)
	                            .serve(service)
	                            .map_err(|e| eprintln!("hyper error: {}", e));

	hyper::rt::run(server);
}
