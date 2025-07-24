use asset_provider::Assets;
use sui::{
	Layable, LayableExt,
	core::{Event, KeyboardEvent, MouseEvent},
};

use crate::world::{
	tile::render::{self, TILE_RENDER_SIZE},
	tilemap::{SIZE, Tilemap},
};

#[derive(Debug, Clone)]
/// world rendering as a component
pub struct WorldRenderer<'a> {
	tilemap: &'a Tilemap,
}
impl<'a> WorldRenderer<'a> {
	pub fn new(tilemap: &'a Tilemap) -> Self {
		Self { tilemap }
	}
}
impl<'a> Layable for WorldRenderer<'a> {
	fn size(&self) -> (i32, i32) {
		let size = SIZE as i32 * TILE_RENDER_SIZE;
		(size, size)
	}
	fn render(&self, d: &mut sui::Handle, det: sui::Details, scale: f32) {
		render::draw_tilemap(d, &self.tilemap, det.x, det.y, scale);
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

/// Singleplayer, self-contained game renderer
#[derive(Clone, Debug)]
pub struct Game {
	tilemap: Tilemap,

	/// camera center position in world coordinates
	// camera_at: (f32, f32), // TODO: wrap the worldrenderer in a view and make moving possible
	scale: f32,
	scale_velocity: f32,
}
impl Game {
	pub fn new<A: Assets + Send + Sync>(
		assets: &A,
		d: &mut sui::Handle,
		thread: &sui::raylib::RaylibThread,
	) -> anyhow::Result<Self> {
		let tilemap = Tilemap::new(assets, d, thread)?;

		Ok(Self {
			tilemap,
			// camera_at: (SIZE as f32 / 2.0, SIZE as f32 / 2.0),
			scale: 1.0,
			scale_velocity: 0.0,
		})
	}
}

impl Layable for Game {
	fn size(&self) -> (i32, i32) {
		let size = TILE_RENDER_SIZE * SIZE as i32;
		(size, size)
	}

	/// we ignore scale
	fn render(&self, d: &mut sui::Handle, det: sui::Details, scale: f32) {
		println!("{}", self.scale);
		let real_scale = (1.1 as f32).powf(self.scale);

		let comp = WorldRenderer::new(&self.tilemap)
			.scale(real_scale)
			.centered();

		comp.render(d, det, scale);
	}

	fn tick(&mut self) {
		self.scale = self.scale + self.scale_velocity;
		self.scale = self.scale.max(0.1).min(60.0);

		self.scale_velocity *= 0.95;
	}

	fn pass_event(
		&mut self,
		event: Event,
		_det: sui::Details,
		_scale: f32,
	) -> Option<sui::core::ReturnEvent> {
		println!("{event:?}");
		match event {
			Event::MouseEvent(MouseEvent::Scroll { amount, .. }) => {
				self.scale_velocity += amount / 6.0
			}
			Event::KeyboardEvent(_, KeyboardEvent::CharPressed('w')) => todo!(),
			_ => {}
		};
		None
	}
}
