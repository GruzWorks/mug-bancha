use hyper::{Body, Request, Response, Server};
use hyper::rt::Future;
use hyper::service::service_fn_ok;

mod route;

fn main() {
	let addr = ([127, 0, 0, 1], 3000).into();

	let service = || { service_fn_ok(route::route) };

	let server = Server::bind(&addr)
	                     .serve(service)
	                     .map_err(|e| eprintln!("server error: {}", e));

	hyper::rt::run(server);
}
