use std::{borrow::Cow, fmt::Debug};

// sys

pub mod tiletex;
pub use tiletex::TileTexture;

pub mod render;

// tiles

mod air;
mod stone;

pub mod tiles {
	use super::*;
	pub use air::*;
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
	Air(Air),
	Stone(Stone),
}
impl Tile for STile {
	fn name(&self) -> Cow<'static, str> {
		match self {
			STile::Air(a) => a.name(),
			STile::Stone(a) => a.name(),
		}
	}
	fn tile_texture_id(&self) -> TileTexture {
		match self {
			STile::Air(a) => a.tile_texture_id(),
			STile::Stone(a) => a.tile_texture_id(),
		}
	}
}
impl Default for STile {
	fn default() -> Self {
		Self::Air(Air)
	}
}
