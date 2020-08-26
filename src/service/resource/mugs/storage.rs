use rand::Rng;
use serde::{Deserialize, Serialize};

use crate::error::{PipelineError, PipelineResult};

#[derive(Clone, Deserialize, Serialize)]
pub struct Mug {
	pub id: i64,
	pub name: String,
	pub lat: f64,
	pub lon: f64,
	pub address: String,
	pub num_mugs: u32,
}

#[derive(Deserialize, Serialize)]
pub struct EphemeralMug {
	pub name: String,
	pub lat: f64,
	pub lon: f64,
	pub address: String,
	pub num_mugs: u32,
}

pub struct Storage {
	data: Vec<Mug>,
}

impl Storage {
	pub fn init() -> Storage {
		Storage { data: Vec::new() }
	}

	pub fn select_all(&self) -> PipelineResult<Vec<Mug>> {
		PipelineResult::Ok(self.data.clone())
	}

	pub fn insert(&mut self, v: EphemeralMug) -> PipelineResult<Mug> {
		let mut id: i64 = rand::thread_rng().gen();
		while self.retrieve(id).is_some() {
			id = rand::thread_rng().gen();
		}
		let entry = Mug {
			id,
			name: v.name,
			lat: v.lat,
			lon: v.lon,
			address: v.address,
			num_mugs: v.num_mugs,
		};
		self.data.push(entry.clone());
		PipelineResult::Ok(entry)
	}

	pub fn update(&mut self, v: Mug) -> PipelineResult<Mug> {
		match self.retrieve(v.id) {
			Some(entry) => {
				*entry = v;
				PipelineResult::Ok(entry.clone())
			}
			None => {
				PipelineResult::Err(PipelineError::NotFound(String::from("Mug does not exist")))
			}
		}
	}

	pub fn delete(&mut self, v: Mug) -> PipelineResult<()> {
		let before = self.data.len();
		self.data.retain(|entry| entry.id != v.id);
		if self.data.len() < before {
			PipelineResult::Ok(())
		} else {
			PipelineResult::Err(PipelineError::NotFound(String::from("Mug does not exist")))
		}
	}

	fn retrieve(&mut self, id: i64) -> Option<&mut Mug> {
		let mut mug = None;
		for entry in &mut self.data {
			if entry.id == id {
				mug = Some(entry);
				break;
			}
		}
		mug
	}
}
