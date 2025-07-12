use std::borrow::Cow;

use crate::world::tile::{Tile, TileTexture};

#[derive(Copy, Clone, Debug, Default)]
pub struct Air;
impl Tile for Air {
	fn name(&self) -> Cow<'static, str> {
		"air".into()
	}
	fn tile_texture_id(&self) -> TileTexture {
		TileTexture::None
	}
}
