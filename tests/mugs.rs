use std::sync::Mutex;

use http::{Request, Response};

use mug_bancha::service::{
	self,
	resource::mugs::{self, EphemeralMug, Mug, Storage},
};

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
fn lists_mugs() {
	init_storage();
}
