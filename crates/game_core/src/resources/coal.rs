use std::borrow::Cow;

use crate::{EResource, Resource, tile::Tile};
use textures::TextureID;

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub struct Coal;
impl Resource for Coal {
	fn name(&self) -> Cow<'static, str> {
		"coal".into()
	}
	fn texture_id(&self) -> TextureID {
		TextureID::Coal
	}
}

#[derive(
	Copy,
	Clone,
	Debug,
	Hash,
	Default,
	bincode::Encode,
	bincode::Decode,
	serde::Serialize,
	serde::Deserialize,
)]
pub struct CoalOre;
impl Tile for CoalOre {
	fn name(&self) -> Cow<'static, str> {
		"coal ore".into()
	}
	fn texture_id(&self) -> TextureID {
		TextureID::CoalOre
	}

	fn generate_resource(&self) -> Option<super::EResource> {
		Some(EResource::coal())
	}
}
