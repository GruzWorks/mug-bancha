use async_trait::async_trait;

use crate::service::resource::mugs::{EphemeralMug, Mug};

pub mod test_storage;

pub type Result<T> = std::result::Result<T, StorageErr>;

pub struct StorageErr;

#[async_trait]
pub trait Storage: Clone + Send {
	async fn select_all_mugs(&self) -> Result<Vec<Mug>>;
	async fn insert_mug(&self, v: EphemeralMug) -> Result<Mug>;
	async fn update_mug(&self, v: Mug) -> Result<Mug>;
	async fn delete(&self, v: Mug) -> Result<()>;
}
