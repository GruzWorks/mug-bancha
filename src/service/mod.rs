use std::sync::Mutex;

use futures::Future;
use serde::{Deserialize, Serialize};

use crate::route;
use resource::mugs;

pub mod resource;

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

	let service = || hyper::service::service_fn(&route::route_request);

	let server = hyper::Server::bind(&addr)
		.serve(service)
		.map_err(|e| eprintln!("hyper error: {}", e));

	hyper::rt::run(server);
}

#[cfg(test)]
mod tests {
	use http::{Method, Request, Response, StatusCode};
	use hyper::Body;
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

		let response = handle(request);

		assert_eq!(response.status(), StatusCode::OK);
		let result = deserialize_response::<Message>(response)?;
		assert_eq!(result, Message::from("mug-bancha says hello!"));

		Ok(())
	}

	#[test]
	fn lists_mugs() -> Result<(), String> {
		init_storage();

		let request = request!(GET "/1/mugs");

		let response = handle(request);

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

		let response = handle(request);

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

		let response = handle(request);

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

		let response = handle(request);

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

		let response = handle(request);

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

		let response = handle(request);

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
	fn removes_mug() -> Result<(), String> {
		init_storage();

		let request = request!(GET "/1/mugs");

		let response = handle(request);

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

		let request = request!(DELETE "/1/mugs",
			&Mug {
				id,
				name: String::from("Foo"),
				lat: 51.0,
				lon: 17.0,
				address: String::from("14 Bar Street"),
				num_mugs: 2,
			}
		);

		let response = handle(request);

		assert_eq!(response.status(), StatusCode::OK);
		let _result: () = deserialize_response(response)?;

		let request = request!(GET "/1/mugs");

		let response = handle(request);

		assert_eq!(response.status(), StatusCode::OK);
		let result: Vec<Mug> = deserialize_response(response)?;
		assert_eq!(result, Vec::<Mug>::new());

		Ok(())
	}

	#[test]
	fn invalid_path() -> Result<(), String> {
		let request = request!(GET "/echo");

		let response = handle(request);

		assert_eq!(response.status(), StatusCode::NOT_FOUND);
		let result = deserialize_response::<Message>(response)?;
		assert_eq!(result, Message::from("Not found"));

		Ok(())
	}

	fn handle(request: Request<Body>) -> Response<Body> {
		route::route_request(request).wait().unwrap()
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
