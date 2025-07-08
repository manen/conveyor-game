use std::borrow::Cow;

use crate::tile::{Tile, TileTexture};

#[derive(Copy, Clone, Debug)]
pub struct Stone;
impl Tile for Stone {
	fn name(&self) -> Cow<'static, str> {
		"stone".into()
	}
	fn tile_texture_id(&self) -> TileTexture {
		TileTexture::Stone
	}
}
