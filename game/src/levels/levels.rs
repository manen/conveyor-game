use anyhow::Context;
use asset_provider::Assets;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Levels {
	pub campaign: Campaign,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Campaign {
	pub tutorial: String,
}

impl Levels {
	pub async fn load<A: Assets>(assets: &A) -> anyhow::Result<Self> {
		let asset = assets
			.asset("levels/levels.toml")
			.await
			.with_context(|| format!("while reading levels/levels.toml from assets"))?;

		let levels: Levels = toml::from_slice(asset.as_slice())
			.with_context(|| format!("while parsing levels/level.toml from assets"))?;
		Ok(levels)
	}
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct LevelMetadata {
	pub level: LevelMetadataInner,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct LevelMetadataInner {
	pub name: String,
	pub desc: String,
}

impl LevelMetadata {
	pub async fn load<A: Assets>(assets: &A, id: &str) -> anyhow::Result<Self> {
		let metadata_path = format!("levels/{id}/level.toml");
		let asset = assets
			.asset(&metadata_path)
			.await
			.with_context(|| format!("while reading level metadata from {metadata_path}"))?;

		let metadata: LevelMetadata = toml::from_slice(asset.as_slice())
			.with_context(|| format!("while parsing level metadata at {metadata_path}"))?;
		Ok(metadata)
	}
}
