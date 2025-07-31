use std::path::Path;

use anyhow::{Context, anyhow};
use bincode::{Decode, Encode};

use crate::world::{
	ETile,
	tilemap::{SIZE, Tilemap},
};

#[derive(Copy, Clone, Debug, PartialEq, Eq, Encode, Decode, Default)]
pub enum SaveFormat {
	#[default]
	Early,
}

#[derive(Clone, Debug, Encode, Decode)]
pub struct Level {
	format: SaveFormat,
	tiles: Vec<Vec<ETile>>,
}
impl Level {
	pub async fn load<P: AsRef<Path> + Send + 'static>(path: P) -> anyhow::Result<Self> {
		let path = path.as_ref();
		let file = tokio::fs::OpenOptions::new()
			.read(true)
			.open(path)
			.await
			.with_context(|| format!("while loading Level from {}", path.display()))?;
		let mut file = file.into_std().await;

		let decoded: Self = bincode::decode_from_std_read(&mut file, bincode::config::standard())?;
		Ok(decoded)
	}
	pub async fn save<P: AsRef<Path> + Send + 'static>(&self, path: P) -> anyhow::Result<()> {
		let path = path.as_ref();
		let file = tokio::fs::OpenOptions::new()
			.create(true)
			.write(true)
			.open(path)
			.await
			.with_context(|| format!("while saving level to {}", path.display()))?;
		let mut file = file.into_std().await;

		bincode::encode_into_std_write(self, &mut file, bincode::config::standard())?;
		Ok(())
	}

	pub fn from_tilemap(tilemap: &Tilemap) -> Self {
		let tiles = tilemap
			.tiles()
			.iter()
			.map(|a| a.iter().cloned().collect::<Vec<_>>())
			.collect::<Vec<_>>();

		Self {
			format: SaveFormat::default(),
			tiles,
		}
	}
	pub fn into_tilemap(&self) -> anyhow::Result<Tilemap> {
		if self.tiles.len() != SIZE {
			return Err(anyhow!(
				"loaded level isn't {SIZE}x{SIZE} (len: {})\n(level: {self:?})",
				self.tiles.len()
			));
		}
		if self
			.tiles
			.iter()
			.map(|a| a.len())
			.filter(|len| *len != SIZE)
			.count() > 0
		{
			return Err(anyhow!(
				"loaded level's columns aren't {SIZE} long\n(level: {self:?})"
			));
		}

		let tiles = core::array::from_fn(|x| core::array::from_fn(|y| self.tiles[x][y].clone()));
		let tilemap = Tilemap::from_tiles(tiles);

		Ok(tilemap)
	}
}
