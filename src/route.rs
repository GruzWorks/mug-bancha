use hyper::{Body, Method, Request, Response, Server};

pub fn route(request: Request<Body>) -> Response<Body> {
	match request.uri().path() {
		"/" => match request.method() {
			&Method::GET => Response::new(Body::from("INCREDIBLE")),
			_ => Response::builder()
			               .status(403)
			               .body(Body::empty())
			               .unwrap(),
		},
		_ => Response::builder()
		               .status(404)
		               .body(Body::empty())
		               .unwrap(),
	}
}
