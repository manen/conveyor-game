use std::{
	fmt::{Debug, Display},
	io::{Read, Seek, Write},
};

use anyhow::{Context, anyhow};

use crate::{
	EResource, ETile, Tile,
	buildings::EBuilding,
	maps::{BuildingsMap, Tilemap, TilemapExt},
};

#[derive(Clone, Debug)]
pub struct GameData {
	pub tilemap: Tilemap,
	pub buildings: BuildingsMap,
}
impl GameData {
	pub fn new(tilemap: Tilemap, buildings: BuildingsMap) -> Self {
		Self { tilemap, buildings }
	}

	pub fn tile_resource_at(&self, pos: (i32, i32)) -> Option<EResource> {
		let tile = self.tilemap.at(pos)?;
		let resource = tile.generate_resource();
		resource
	}

	pub fn world_size(&self) -> (usize, usize) {
		self.tilemap.size()
	}

	pub fn tick(&mut self) {
		let tile_resource_at = |pos| {
			let tile = self.tilemap.at(pos)?;
			let resource = tile.generate_resource();
			resource
		};
		self.buildings.tick(tile_resource_at);
	}
}

// -

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct GameDataSave(pub Vec<Vec<(ETile, EBuilding)>>);
impl GameDataSave {
	pub fn new(game_data: &GameData) -> anyhow::Result<Self> {
		let (w, h) = game_data.tilemap.size();

		let save = (0..w)
			.map(|x| {
				(0..h)
					.map(|y| {
						0;

						let tile = game_data.tilemap.at_usize((x, y)).cloned();
						let tile = tile.with_context(|| {
							format!("tilemap is reportedly bigger than it actually is: {x}, {y}")
						})?;

						let building = game_data.buildings.at((x as _, y as _)).cloned();
						let building = building.with_context(|| {
							format!("buildingsmap is smaller than it reported: {x}, {y}")
						})?;
						let building = match building {
							EBuilding::ChannelConsumer(_) => EBuilding::nothing(),
							a => a,
						};

						anyhow::Ok((tile, building))
					})
					.collect::<anyhow::Result<Vec<_>>>()
			})
			.collect::<anyhow::Result<Vec<_>>>();
		let save = save.with_context(|| format!("while building GameDataSave from GameData"))?;

		Ok(Self(save))
	}
	pub fn take(self) -> anyhow::Result<GameData> {
		let (w, h) = (
			self.0.len(),
			self.0.iter().nth(0).map(|a| a.len()).unwrap_or_default(),
		);

		let mut tilemap = Tilemap::stone(w, h);
		let mut buildings = BuildingsMap::new(w, h);

		for (x, entry) in self.0.into_iter().enumerate() {
			for (y, (tile, building)) in entry.into_iter().enumerate() {
				if let Some(tile_location) = tilemap.at_mut_usize((x, y)) {
					*tile_location = tile;
				} else {
					return Err(anyhow!(
						"the tilemap we just created doesn't work: {x}, {y}"
					));
				};
				if let Some(building_location) = buildings.at_mut((x as _, y as _)) {
					*building_location = building;
				} else {
					return Err(anyhow!(
						"the buildingsmap we just created doesn't work: {x}, {y}"
					));
				}
			}
		}

		Ok(GameData::new(tilemap, buildings))
	}

	pub fn save<W: Write>(&self, write: &mut W) -> anyhow::Result<()> {
		serde_cbor::to_writer(write, self)
			.with_context(|| format!("while serializing save file"))?;
		Ok(())
	}

	pub fn load<R: Read>(read: &mut R) -> anyhow::Result<Self> {
		let deser = serde_cbor::from_reader(read)
			.with_context(|| format!("while deserializing cbor save file"))?;
		Ok(deser)
	}
	pub fn load_bincode<R: Read>(read: &mut R) -> anyhow::Result<Self> {
		let decoded: GameDataSave =
			bincode::serde::decode_from_std_read(read, bincode::config::standard())
				.with_context(|| format!("while deserializing bincode save file"))?;
		Ok(decoded)
	}
	pub fn load_as_either<R: Read + Seek>(read: &mut R) -> anyhow::Result<Self> {
		struct BothFailedError {
			cbor: anyhow::Error,
			bincode: anyhow::Error,
		}
		impl Display for BothFailedError {
			fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
				write!(f, "failed to load save file in either format")
			}
		}
		impl Debug for BothFailedError {
			fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
				writeln!(f, "cbor error: {:?}\n", self.cbor)?;
				writeln!(f, " -- \n")?;
				writeln!(f, "bincode error: {:?}\n", self.bincode)
			}
		}
		impl std::error::Error for BothFailedError {}

		let decoded = match GameDataSave::load(read) {
			Ok(cbor) => Ok(cbor),
			Err(cbor_err) => {
				read.seek(std::io::SeekFrom::Start(0))?;
				match GameDataSave::load_bincode(read) {
					Ok(bincode) => Ok(bincode),
					Err(bincode_err) => Err(BothFailedError {
						cbor: cbor_err,
						bincode: bincode_err,
					}),
				}
			}
		}?;
		Ok(decoded)
	}
}
