use crate::{
	textures::TextureID,
	world::{Resource, Tile},
};

pub struct RawIron;
impl Resource for RawIron {
	fn name(&self) -> std::borrow::Cow<'static, str> {
		"raw iron".into()
	}
	fn texture_id(&self) -> TextureID {
		TextureID::RawIron
	}
}

use std::borrow::Cow;

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
