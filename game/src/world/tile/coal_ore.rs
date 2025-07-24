use std::borrow::Cow;

use crate::world::tile::{Tile, TileTexture};

#[derive(Copy, Clone, Debug)]
pub struct CoalOre;
impl Tile for CoalOre {
	fn name(&self) -> Cow<'static, str> {
		"coal ore".into()
	}
	fn tile_texture_id(&self) -> TileTexture {
		TileTexture::CoalOre
	}
}
