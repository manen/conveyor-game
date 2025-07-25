use sui::Layable;

use crate::{
	textures::Textures,
	world::{ETile, tile::render::TILE_RENDER_SIZE, worldgen},
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

	pub fn tiles(&self) -> &[[ETile; SIZE]; SIZE] {
		&self.tiles
	}
	pub fn tiles_mut(&mut self) -> &mut [[ETile; SIZE]; SIZE] {
		&mut self.tiles
	}

	pub fn render<'a, 'b: 'a>(&'a self, textures: &'b Textures) -> TilemapRenderer<'a> {
		TilemapRenderer::new(self, textures)
	}

	pub fn at(&self, (x, y): (usize, usize)) -> Option<&ETile> {
		if x > SIZE - 1 {
			if x > SIZE - 1 {
				return None;
			}
		}

		Some(&self.tiles[x][y])
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
		crate::world::tile::render::draw_tilemap(
			d,
			&self.tilemap,
			&self.textures,
			det.x,
			det.y,
			scale,
		);
	}

	fn pass_event(
		&mut self,
		_event: sui::core::Event,
		_det: sui::Details,
		_scale: f32,
	) -> Option<sui::core::ReturnEvent> {
		None
	}
}
