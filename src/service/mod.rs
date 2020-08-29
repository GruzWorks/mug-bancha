use std::sync::Mutex;

use futures::future;
use futures::future::Future;
use http::{Request, StatusCode};
use hyper::{Body, Method};
use serde::{Deserialize, Serialize};
use serde_json::json;

use crate::route;
use crate::route::{Endpoint, Router};
use resource::echo;
use resource::mugs;

mod resource;

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

pub fn run() {
	unsafe {
		mugs::MUGS.storage = Some(Mutex::new(mugs::Storage::init()));
	}

	let addr = ([127, 0, 0, 1], 3000).into();

	let service = || hyper::service::service_fn(&handle_request);

	let server = hyper::Server::bind(&addr)
		.serve(service)
		.map_err(|e| eprintln!("hyper error: {}", e));

	hyper::rt::run(server);
}

fn handle_request(request: Request<Body>) -> route::BoxOfDreams {
	let (parts, body) = request.into_parts();
	match (parts.uri.path(), parts.method) {
		("/1/echo", Method::GET) => route::process_request(body, &echo::get),
		("/1/mugs", Method::GET) => route::process_request(body, &mugs::get),
		("/1/mugs", Method::PUT) => route::process_request(body, &mugs::put),
		("/1/mugs", Method::PATCH) => route::process_request(body, &mugs::patch),
		("/1/mugs", Method::DELETE) => route::process_request(body, &mugs::delete),
		_ => Box::new(future::ok(route::error_response(
			StatusCode::NOT_FOUND,
			Message::from("Not found"),
		))),
	}
	/*
	Router::with_routes(vec![
		route!(Endpoint("/1/echo", &Method::GET),
			   &echo::get,
			   (),
			   Message,
			   "",
			   r#"{"message":"mug-bancha says hello!"}"#),
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
	]).dispatch(request)*/
}

#[cfg(test)]
mod tests {
	use bytes::Bytes;
	use futures::stream::Stream;
	use http::Response;
	use serde::de::DeserializeOwned;

	use super::*;
	use resource::mugs::{self, EphemeralMug, Mug, Storage};

	macro_rules! request {
		( $method:ident $path:expr ) => {
			Request::builder()
				.uri(format!("http://foo.bar{}", $path))
				.method(Method::$method)
				.body(Body::empty())
				.unwrap()
		};
		( $method:ident $path:expr, $payload:expr ) => {
			Request::builder()
				.uri(format!("http://foo.bar{}", $path))
				.method(Method::$method)
				.body(Body::from(serde_json::to_string($payload).unwrap()))
				.unwrap()
		};
	}

	fn init_storage() {
		let mut storage = Storage::init();

		storage
			.insert(EphemeralMug {
				name: String::from("Foo"),
				lat: 51.0,
				lon: 17.0,
				address: String::from("14 Bar Street"),
				num_mugs: 2,
			})
			.unwrap();

		unsafe {
			mugs::MUGS.storage = Some(Mutex::new(storage));
		}
	}

	#[test]
	fn echoes() -> Result<(), String> {
		let request = request!(GET "/1/echo");

		let response = handle_request(request).wait().unwrap();

		assert_eq!(response.status(), StatusCode::OK);
		let result = deserialize_response::<Message>(response)?;
		assert_eq!(result, Message::from("mug-bancha says hello!"));

		Ok(())
	}

	#[test]
	fn lists_mugs() -> Result<(), String> {
		init_storage();

		let request = request!(GET "/1/mugs");

		let response = handle_request(request).wait().unwrap();

		assert_eq!(response.status(), StatusCode::OK);
		let result = deserialize_response::<Vec<Mug>>(response)?;
		assert_eq!(
			result,
			vec![EphemeralMug {
				name: String::from("Foo"),
				lat: 51.0,
				lon: 17.0,
				address: String::from("14 Bar Street"),
				num_mugs: 2,
			}]
		);

		Ok(())
	}

	#[test]
	fn inserts_mug() -> Result<(), String> {
		init_storage();

		let request = request!(PUT "/1/mugs",
			&EphemeralMug {
				name: String::from("Point"),
				lat: -39.0,
				lon: -67.0,
				address: String::from("Real Address"),
				num_mugs: 4,
			}
		);

		let response = handle_request(request).wait().unwrap();

		assert_eq!(response.status(), StatusCode::OK);
		let result: Mug = deserialize_response(response)?;
		assert_eq!(
			result,
			EphemeralMug {
				name: String::from("Point"),
				lat: -39.0,
				lon: -67.0,
				address: String::from("Real Address"),
				num_mugs: 4,
			}
		);

		let request = request!(GET "/1/mugs");

		let response = handle_request(request).wait().unwrap();

		assert_eq!(response.status(), StatusCode::OK);
		let result = deserialize_response::<Vec<Mug>>(response)?;
		assert_eq!(
			result,
			vec![
				EphemeralMug {
					name: String::from("Foo"),
					lat: 51.0,
					lon: 17.0,
					address: String::from("14 Bar Street"),
					num_mugs: 2,
				},
				EphemeralMug {
					name: String::from("Point"),
					lat: -39.0,
					lon: -67.0,
					address: String::from("Real Address"),
					num_mugs: 4,
				},
			]
		);

		Ok(())
	}

	#[test]
	fn updates_mug() -> Result<(), String> {
		init_storage();

		let request = request!(GET "/1/mugs");

		let response = handle_request(request).wait().unwrap();

		assert_eq!(response.status(), StatusCode::OK);
		let result: Vec<Mug> = deserialize_response(response)?;
		assert_eq!(
			result,
			vec![EphemeralMug {
				name: String::from("Foo"),
				lat: 51.0,
				lon: 17.0,
				address: String::from("14 Bar Street"),
				num_mugs: 2,
			}]
		);

		let id = result[0].id;

		let request = request!(PATCH "/1/mugs",
			&Mug {
				id,
				name: String::from("Foo Baz"),
				lat: 52.01,
				lon: 16.93,
				address: String::from("14 Bar Street"),
				num_mugs: 3,
			}
		);

		let response = handle_request(request).wait().unwrap();

		assert_eq!(response.status(), StatusCode::OK);
		let result: Mug = deserialize_response(response)?;
		assert_eq!(
			result,
			Mug {
				id,
				name: String::from("Foo Baz"),
				lat: 52.01,
				lon: 16.93,
				address: String::from("14 Bar Street"),
				num_mugs: 3,
			}
		);

		let request = request!(GET "/1/mugs");

		let response = handle_request(request).wait().unwrap();

		assert_eq!(response.status(), StatusCode::OK);
		let result: Vec<Mug> = deserialize_response(response)?;
		assert_eq!(
			result,
			vec![EphemeralMug {
				name: String::from("Foo Baz"),
				lat: 52.01,
				lon: 16.93,
				address: String::from("14 Bar Street"),
				num_mugs: 3,
			}]
		);

		Ok(())
	}

	#[test]
	fn invalid_path() -> Result<(), String> {
		let request = request!(GET "/echo");

		let response = handle_request(request).wait().unwrap();

		assert_eq!(response.status(), StatusCode::NOT_FOUND);
		let result = deserialize_response::<Message>(response)?;
		assert_eq!(result, Message::from("Not found"));

		Ok(())
	}

	fn deserialize_response<T: DeserializeOwned>(response: Response<Body>) -> Result<T, String> {
		route::json_bytes(response.into_body())
			.map(move |chunk| serde_json::from_slice::<T>(&chunk))
			.wait()
			.unwrap()
			.map_err(|e| {
				format!(
					"Reponse body did not match type {}: {}",
					std::any::type_name::<T>(),
					e
				)
			})
	}
}
