use sui::Layable;

use crate::{
	textures::Textures,
	world::{ETile, render::TILE_RENDER_SIZE, worldgen},
};

/// world size in tiles
pub const SIZE: usize = 32;

pub type Tilemap = super::Map<ETile>;

pub trait TilemapExt {
	fn new(width: usize, height: usize) -> Self;
	fn stone(width: usize, height: usize) -> Self;
	fn from_tiles<const SIZE: usize>(tiles: [[ETile; SIZE]; SIZE]) -> Self;

	fn render<'a, 'b: 'a>(&'a self, textures: &'b Textures) -> TilemapRenderer<'a>;
}

impl TilemapExt for Tilemap {
	fn new(width: usize, height: usize) -> Self {
		Self::from_vec(worldgen::gen_tiles(width, height))
			.expect("this shouldn't be possible TilemapExt::new")
	}
	fn stone(width: usize, height: usize) -> Self {
		let map = (0..width)
			.map(|_| (0..height).map(|_| ETile::stone()).collect())
			.collect();
		Self::from_vec(map).expect("this shouldn't be possible TilemapExt::stone")
	}
	fn from_tiles<const SIZE: usize>(tiles: [[ETile; SIZE]; SIZE]) -> Self {
		let map = Tilemap::from_vec(tiles.into_iter().map(|a| a.into_iter().collect()).collect())
			.expect("this is impossible thanks to the type system");

		map
	}

	fn render<'a, 'b: 'a>(&'a self, textures: &'b Textures) -> TilemapRenderer<'a> {
		TilemapRenderer::new(self, textures, self.width, self.height)
	}
}

#[derive(Clone, Debug)]
/// world rendering as a component
pub struct TilemapRenderer<'a> {
	width: usize,
	height: usize,

	textures: &'a Textures,
	tilemap: &'a Tilemap,
}
impl<'a> TilemapRenderer<'a> {
	pub fn new(tilemap: &'a Tilemap, textures: &'a Textures, width: usize, height: usize) -> Self {
		Self {
			tilemap,
			textures,
			width,
			height,
		}
	}
}
impl<'a> Layable for TilemapRenderer<'a> {
	fn size(&self) -> (i32, i32) {
		let size = SIZE as i32 * TILE_RENDER_SIZE;
		(size, size)
	}
	fn render(&self, d: &mut sui::Handle, det: sui::Details, scale: f32) {
		crate::world::render::draw_tilemap(
			d,
			&self.tilemap,
			&self.textures,
			det.x,
			det.y,
			scale,
			self.width,
			self.height,
		);
	}
}
