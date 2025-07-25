use asset_provider::Assets;
use sui::{
	Layable, LayableExt,
	core::{Event, KeyboardEvent, MouseEvent},
};

use crate::{
	textures::Textures,
	world::{
		tile::render::{self, TILE_RENDER_SIZE},
		tilemap::{SIZE, Tilemap},
		worldgen,
	},
};

/// Singleplayer, self-contained game renderer
#[derive(Debug)]
pub struct Game {
	textures: Textures,

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
		let textures = Textures::new(assets, d, thread)?;
		let tilemap = Tilemap::new();

		Ok(Self {
			textures,
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
		let real_scale = (1.1 as f32).powf(self.scale);

		let comp = self
			.tilemap
			.render(&self.textures)
			.scale(real_scale)
			.centered();

		comp.render(d, det, scale);
	}

	fn tick(&mut self) {
		self.scale = self.scale + self.scale_velocity;
		self.scale = self.scale.max(-40.0).min(60.0);

		self.scale_velocity *= 0.95;
		if self.scale_velocity.abs() < 0.005 {
			self.scale_velocity = 0.0;
		}
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
			Event::KeyboardEvent(_, KeyboardEvent::CharPressed('r')) => {
				*self.tilemap.tiles_mut() = worldgen::gen_tiles();
			}
			_ => {}
		};
		None
	}
}
