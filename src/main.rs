use hyper::{Server};
use hyper::rt::Future;
use hyper::service::service_fn;

mod route;

fn main() {
	let addr = ([127, 0, 0, 1], 3000).into();

	let service = || { service_fn(route::dispatch) };

	let server = Server::bind(&addr)
	                     .serve(service)
	                     //.into()
	                     .map_err(|e| eprintln!("server error: {}", e));

	hyper::rt::run(server);
}
