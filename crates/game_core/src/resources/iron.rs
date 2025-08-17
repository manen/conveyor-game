use crate::{EResource, Resource, Tile};
use textures::TextureID;

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub struct RawIron;
impl Resource for RawIron {
	fn name(&self) -> std::borrow::Cow<'static, str> {
		"raw iron".into()
	}
	fn texture_id(&self) -> TextureID {
		TextureID::RawIron
	}
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub struct Iron;
impl Resource for Iron {
	fn name(&self) -> std::borrow::Cow<'static, str> {
		"iron".into()
	}
	fn texture_id(&self) -> TextureID {
		TextureID::Iron
	}
}

use std::borrow::Cow;

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
pub struct IronOre;
impl Tile for IronOre {
	fn name(&self) -> Cow<'static, str> {
		"iron ore".into()
	}
	fn texture_id(&self) -> TextureID {
		TextureID::IronOre
	}

	fn generate_resource(&self) -> Option<super::EResource> {
		Some(EResource::raw_iron())
	}
}
