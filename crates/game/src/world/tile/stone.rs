use std::borrow::Cow;

use crate::{textures::TextureID, world::tile::Tile};

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
pub struct Stone;
impl Tile for Stone {
	fn name(&self) -> Cow<'static, str> {
		"stone".into()
	}
	fn texture_id(&self) -> TextureID {
		TextureID::Stone
	}
}
