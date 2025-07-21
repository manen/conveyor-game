use asset_provider::Assets;
use sui::Layable;

use crate::world::{
	tile::render::{self, TILE_RENDER_SIZE},
	tilemap::{SIZE, Tilemap},
};

/// Singleplayer, self-contained game renderer
#[derive(Clone, Debug)]
pub struct Game {
	tilemap: Tilemap,
}
impl Game {
	pub fn new<A: Assets + Send + Sync>(
		assets: &A,
		d: &mut sui::Handle,
		thread: &sui::raylib::RaylibThread,
	) -> anyhow::Result<Self> {
		let tilemap = Tilemap::new(assets, d, thread)?;
		Ok(Self { tilemap })
	}
}

impl Layable for Game {
	fn size(&self) -> (i32, i32) {
		let size = TILE_RENDER_SIZE * SIZE as i32;
		(size, size)
	}
	fn render(&self, d: &mut sui::Handle, det: sui::Details, scale: f32) {
		render::draw_tilemap(d, &self.tilemap, det.x, det.y, scale);
	}

	fn pass_event(
		&self,
		event: sui::core::Event,
		_det: sui::Details,
		_scale: f32,
	) -> Option<sui::core::ReturnEvent> {
		println!("{event:?}");
		None
	}
}
