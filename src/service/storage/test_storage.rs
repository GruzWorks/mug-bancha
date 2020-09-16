use std::sync::Arc;

use async_trait::async_trait;
use rand::Rng;
use tokio::sync::{RwLock, RwLockWriteGuard};

use super::{Result, Storage, StorageErr};
use crate::service::resource::mugs::{EphemeralMug, Mug};

type Access<T> = Arc<RwLock<T>>;

#[derive(Clone)]
pub struct TestStorage {
	mugs: Access<Vec<Mug>>,
}

impl TestStorage {
	pub fn new() -> Self {
		Self {
			mugs: Arc::new(RwLock::new(Vec::new())),
		}
	}
}

#[async_trait]
impl Storage for TestStorage {
	async fn select_all_mugs(&self) -> Result<Vec<Mug>> {
		Ok(self.mugs.read().await.clone())
	}

	async fn insert_mug(&self, v: EphemeralMug) -> Result<Mug> {
		let mut id: i64 = rand::thread_rng().gen();
		let mut write = self.mugs.write().await;
		while retrieve_mug_mut(&mut write, id).is_some() {
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
		write.push(entry.clone());
		Ok(entry)
	}

	async fn update_mug(&self, v: Mug) -> Result<Mug> {
		let mut write = self.mugs.write().await;
		match retrieve_mug_mut(&mut write, v.id) {
			Some(entry) => {
				*entry = v;
				Ok(entry.clone())
			}
			None => Err(StorageErr),
			//None => Err(PipelineError::NotFound(String::from("Mug does not exist"))),
		}
	}

	async fn delete(&self, v: Mug) -> Result<()> {
		let mut write = self.mugs.write().await;
		let before = write.len();
		write.retain(|entry| entry.id != v.id);
		if write.len() < before {
			Ok(())
		} else {
			Err(StorageErr)
			//PipelineResult::Err(PipelineError::NotFound(String::from("Mug does not exist")))
		}
	}
}

fn retrieve_mug_mut<'a>(
	write: &'a mut RwLockWriteGuard<'_, Vec<Mug>>,
	id: i64,
) -> Option<&'a mut Mug> {
	let mut mug = None;
	for entry in write.iter_mut() {
		if entry.id == id {
			mug = Some(entry);
			break;
		}
	}
	mug
}
