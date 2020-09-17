use std::sync::Mutex;

use crate::{
	error::{PipelineError, PipelineResult},
	service::storage::Storage,
};
pub use storage::{EphemeralMug, Mug};

mod storage;
/*
pub struct Mugs {
	pub storage: Option<Mutex<Storage>>,
}

pub static mut MUGS: Mugs = Mugs { storage: None };
*/
pub async fn get(_: (), storage: Box<dyn Storage>) -> PipelineResult<Vec<Mug>> {
	storage
		.select_all_mugs()
		.await
		.map_err(|_| PipelineError::CannotFulfil)
}

pub async fn put(mug: EphemeralMug, storage: Box<dyn Storage>) -> PipelineResult<Mug> {
	storage
		.insert_mug(mug)
		.await
		.map_err(|_| PipelineError::CannotFulfil)
}

pub async fn patch(mug: Mug, storage: Box<dyn Storage>) -> PipelineResult<Mug> {
	storage
		.update_mug(mug)
		.await
		.map_err(|_| PipelineError::CannotFulfil)
}

pub async fn delete(mug: Mug, storage: Box<dyn Storage>) -> PipelineResult<()> {
	storage
		.delete_mug(mug)
		.await
		.map_err(|_| PipelineError::CannotFulfil)
}
