use std::path::PathBuf;

use anyhow::{Context, anyhow};
use asset_provider::{Asset, Assets};

use include_dir::{Dir, include_dir};
static ASSETS: Dir = include_dir!("$CARGO_MANIFEST_DIR/../../assets");

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
	async fn asset(&self, key: &str) -> anyhow::Result<Asset> {
		let baked_in = || {
			let res = ASSETS
				.get_file(key)
				.ok_or_else(|| anyhow!("couldn't retreive {key} from baked-in asset storage"))?;

			anyhow::Ok(Asset::new(res.contents()))
		};
		let web = async || {
			let res = self.client.get(key).send().await?;
			let bytes = res.bytes().await?;

			anyhow::Ok(Asset::new(Vec::from(bytes)))
		};
		let fs = async || {
			let path = self.run_dir.join(key);
			let bytes = std::fs::read(&path)?;

			anyhow::Ok(Asset::new(bytes))
		};

		let is_url = match key.strip_prefix("http") {
			Some(a) if a.starts_with("://") || a.starts_with("s://") => true,
			_ => false,
		};
		if is_url {
			Ok(web()
				.await
				.with_context(|| format!("while resolving asset {key}"))?)
		} else {
			match fs().await {
				Ok(a) => Ok(a),
				Err(fs_err) => match baked_in() {
					Ok(a) => Ok(a),
					Err(baked_in_err) => {
						return Err(anyhow!(
							"couldn't retreive asset {key}\nfilesystem error: {fs_err}\nbaked-in storage error: {baked_in_err}"
						));
					}
				},
			}
		}
	}
}
