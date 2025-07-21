use std::{borrow::Cow, fmt::Debug};

// sys

pub mod tiletex;
pub use tiletex::TileTexture;

pub mod render;

// tiles

mod stone;

pub mod tiles {
	use super::*;
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
}
impl Tile for STile {
	fn name(&self) -> Cow<'static, str> {
		match self {
			STile::Stone(a) => a.name(),
		}
	}
	fn tile_texture_id(&self) -> TileTexture {
		match self {
			STile::Stone(a) => a.tile_texture_id(),
		}
	}
}
