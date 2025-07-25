use std::borrow::Cow;

use crate::{textures::TextureID, world::tile::Tile};

#[derive(Copy, Clone, Debug)]
pub struct IronOre;
impl Tile for IronOre {
	fn name(&self) -> Cow<'static, str> {
		"iron ore".into()
	}
	fn texture_id(&self) -> TextureID {
		TextureID::IronOre
	}
}
