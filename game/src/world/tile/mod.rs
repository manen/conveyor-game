use std::{borrow::Cow, fmt::Debug};

pub mod render;

// tiles

mod stone;

pub mod tiles {
	use super::*;
	use crate::world::resources;

	pub use resources::CoalOre;
	pub use resources::IronOre;
	pub use stone::*;
}
use tiles::*;

use crate::textures::TextureID;

pub trait Tile: Clone + Debug {
	fn name(&self) -> Cow<'static, str>;
	fn texture_id(&self) -> TextureID;
}

/// tile enum contains the vanilla tiles for performance and ease of use
#[derive(Clone, Debug)]
pub enum ETile {
	Stone(Stone),
	IronOre(IronOre),
	CoalOre(CoalOre),
}
impl ETile {
	pub fn stone() -> Self {
		Self::Stone(Stone)
	}
	pub fn iron_ore() -> Self {
		Self::IronOre(IronOre)
	}
	pub fn coal_ore() -> Self {
		Self::CoalOre(CoalOre)
	}
}
impl Tile for ETile {
	fn name(&self) -> Cow<'static, str> {
		match self {
			ETile::Stone(a) => a.name(),
			ETile::CoalOre(a) => a.name(),
			ETile::IronOre(a) => a.name(),
		}
	}
	fn texture_id(&self) -> TextureID {
		match self {
			ETile::Stone(a) => a.texture_id(),
			ETile::IronOre(a) => a.texture_id(),
			ETile::CoalOre(a) => a.texture_id(),
		}
	}
}
