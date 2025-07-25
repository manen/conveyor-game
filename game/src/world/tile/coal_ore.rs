use std::borrow::Cow;

use crate::{textures::TextureID, world::tile::Tile};

#[derive(Copy, Clone, Debug)]
pub struct CoalOre;
impl Tile for CoalOre {
	fn name(&self) -> Cow<'static, str> {
		"coal ore".into()
	}
	fn texture_id(&self) -> TextureID {
		TextureID::CoalOre
	}
}
