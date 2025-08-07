use crate::world::{
	Tile,
	maps::{BuildingsMap, Tilemap},
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

	pub fn tick(&mut self) {
		let tile_resource_at = |pos| {
			let tile = self.tilemap.at(pos)?;
			let resource = tile.generate_resource();
			resource
		};
		self.buildings.tick(tile_resource_at);
	}
}
