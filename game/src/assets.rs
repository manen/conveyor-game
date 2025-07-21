use std::path::PathBuf;

use asset_provider::{Asset, Assets};

pub struct GameAssets {
	client: reqwest::Client,
	run_dir: PathBuf,
}
impl Default for GameAssets {
	fn default() -> Self {
		Self {
			client: reqwest::Client::new(),
			run_dir: std::env::current_dir()
				.expect("couldn't get cwd")
				.join("run"),
		}
	}
}
impl Assets for GameAssets {
	async fn asset(&self, key: &str) -> Result<Asset, asset_provider::Error> {
		let web = async || {
			let res = self.client.get(key).send().await?;
			let bytes = res.bytes().await?;
			return Result::<Asset, asset_provider::Error>::Ok(Asset::new(Vec::from(bytes)));
		};
		let fs = async || {
			let path = self.run_dir.join(key);
			let bytes = std::fs::read(&path)?;

			Result::<Asset, asset_provider::Error>::Ok(Asset::new(bytes))
		};

		let is_url = match key.strip_prefix("http") {
			Some(a) if a.starts_with("://") || a.starts_with("s://") => true,
			_ => false,
		};
		if is_url { web().await } else { fs().await }
	}
}
