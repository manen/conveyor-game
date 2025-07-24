use std::borrow::Cow;

use crate::world::tile::{Tile, TileTexture};

#[derive(Copy, Clone, Debug)]
pub struct IronOre;
impl Tile for IronOre {
	fn name(&self) -> Cow<'static, str> {
		"iron ore".into()
	}
	fn tile_texture_id(&self) -> TileTexture {
		TileTexture::IronOre
	}
}
