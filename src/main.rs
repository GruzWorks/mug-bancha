#[tokio::main]
async fn main() {
	mug_bancha::service::run().await;
}
