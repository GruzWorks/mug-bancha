use std::sync::Mutex;

use crate::error::{PipelineError, PipelineResult};

pub use storage::{EphemeralMug, Mug, Storage};

mod storage;

pub struct Mugs {
	pub storage: Option<Mutex<Storage>>,
}

pub static mut MUGS: Mugs = Mugs { storage: None };

pub fn get(_: ()) -> PipelineResult<Vec<Mug>> {
	let storage = unsafe { &MUGS.storage };
	match storage {
		Some(mutex) => {
			mutex.lock().unwrap().select_all()
		},
		None => PipelineResult::Err(PipelineError::CannotFulfil),
	}
}

pub fn put(mug: EphemeralMug) -> PipelineResult<Mug> {
	let storage = unsafe { &MUGS.storage };
	match storage {
		Some(mutex) => {
			mutex.lock().unwrap().insert(mug)
		},
		None => PipelineResult::Err(PipelineError::CannotFulfil),
	}
}

pub fn patch(mug: Mug) -> PipelineResult<Mug> {
	let storage = unsafe { &MUGS.storage };
	match storage {
		Some(mutex) => {
			mutex.lock().unwrap().update(mug)
		},
		None => PipelineResult::Err(PipelineError::CannotFulfil),
	}
}

pub fn delete(mug: Mug) -> PipelineResult<()> {
	let storage = unsafe { &MUGS.storage };
	match storage {
		Some(mutex) => {
			mutex.lock().unwrap().delete(mug)
		},
		None => PipelineResult::Err(PipelineError::CannotFulfil),
	}
}
