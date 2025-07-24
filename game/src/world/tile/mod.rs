use std::{borrow::Cow, fmt::Debug};

// sys

pub mod tiletex;
pub use tiletex::TileTexture;

pub mod render;

// tiles

mod coal_ore;
mod iron_ore;
mod stone;

pub mod tiles {
	use super::*;
	pub use coal_ore::*;
	pub use iron_ore::*;
	pub use stone::*;
}
use tiles::*;

pub trait Tile: Clone + Debug {
	fn name(&self) -> Cow<'static, str>;
	fn tile_texture_id(&self) -> TileTexture;
}

/// small tile
#[derive(Clone, Debug)]
pub enum STile {
	Stone(Stone),
	IronOre(IronOre),
	CoalOre(CoalOre),
}
impl Tile for STile {
	fn name(&self) -> Cow<'static, str> {
		match self {
			STile::Stone(a) => a.name(),
			STile::CoalOre(a) => a.name(),
			STile::IronOre(a) => a.name(),
		}
	}
	fn tile_texture_id(&self) -> TileTexture {
		match self {
			STile::Stone(a) => a.tile_texture_id(),
			STile::IronOre(a) => a.tile_texture_id(),
			STile::CoalOre(a) => a.tile_texture_id(),
		}
	}
}
