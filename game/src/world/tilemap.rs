use sui::Layable;

use crate::{
	textures::Textures,
	world::{ETile, render::TILE_RENDER_SIZE, worldgen},
};

/// world size in tiles
pub const SIZE: usize = 32;

#[derive(Clone, Debug)]
pub struct Tilemap {
	tiles: [[ETile; SIZE]; SIZE],
}
impl Tilemap {
	pub fn new() -> Self {
		Self {
			tiles: worldgen::gen_tiles(),
		}
	}
	pub fn stone() -> Self {
		let tiles = core::array::from_fn(|_| core::array::from_fn(|_| ETile::stone()));

		Self { tiles }
	}

	pub fn from_tiles(tiles: [[ETile; SIZE]; SIZE]) -> Self {
		Self { tiles }
	}

	pub fn tiles(&self) -> &[[ETile; SIZE]; SIZE] {
		&self.tiles
	}
	pub fn tiles_mut(&mut self) -> &mut [[ETile; SIZE]; SIZE] {
		&mut self.tiles
	}

	pub fn render<'a, 'b: 'a>(&'a self, textures: &'b Textures) -> TilemapRenderer<'a> {
		TilemapRenderer::new(self, textures)
	}

	pub fn at(&self, pos: (i32, i32)) -> Option<&ETile> {
		if pos.0 < 0 || pos.1 < 0 || pos.0 >= SIZE as _ || pos.1 >= SIZE as _ {
			return None;
		}
		Some(&self.tiles[pos.0 as usize][pos.1 as usize])
	}
	pub fn at_mut(&mut self, pos: (i32, i32)) -> Option<&mut ETile> {
		if pos.0 < 0 || pos.1 < 0 || pos.0 >= SIZE as _ || pos.1 >= SIZE as _ {
			return None;
		}
		Some(&mut self.tiles[pos.0 as usize][pos.1 as usize])
	}
}

#[derive(Clone, Debug)]
/// world rendering as a component
pub struct TilemapRenderer<'a> {
	textures: &'a Textures,
	tilemap: &'a Tilemap,
}
impl<'a> TilemapRenderer<'a> {
	pub fn new(tilemap: &'a Tilemap, textures: &'a Textures) -> Self {
		Self { tilemap, textures }
	}
}
impl<'a> Layable for TilemapRenderer<'a> {
	fn size(&self) -> (i32, i32) {
		let size = SIZE as i32 * TILE_RENDER_SIZE;
		(size, size)
	}
	fn render(&self, d: &mut sui::Handle, det: sui::Details, scale: f32) {
		crate::world::render::draw_tilemap(d, &self.tilemap, &self.textures, det.x, det.y, scale);
	}
}
