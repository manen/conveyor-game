use std::borrow::Cow;

use crate::{
	textures::TextureID,
	world::{Resource, tile::Tile},
};

pub struct Coal;
impl Resource for Coal {
	fn name(&self) -> Cow<'static, str> {
		"coal".into()
	}
	fn texture_id(&self) -> TextureID {
		TextureID::Coal
	}
}

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
