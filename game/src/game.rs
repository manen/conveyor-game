use asset_provider::Assets;
use sui::{
	Layable, LayableExt,
	core::{Event, KeyboardEvent, MouseEvent},
};

use crate::{
	textures::Textures,
	world::{
		tile::render::TILE_RENDER_SIZE,
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
	camera_at: (f32, f32),
	camera_velocity: (f32, f32),
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
			camera_at: (SIZE as f32 / 2.0, SIZE as f32 / 2.0),
			camera_velocity: (0.0, 0.0),
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

		let comp = self.tilemap.render(&self.textures).scale(real_scale).view(
			(self.camera_at.0 * TILE_RENDER_SIZE as f32 * real_scale) as i32 - det.aw / 2,
			(self.camera_at.1 * TILE_RENDER_SIZE as f32 * real_scale) as i32 - det.ah / 2,
		);

		comp.render(d, det, scale);
	}

	fn tick(&mut self) {
		self.camera_at.0 += self.camera_velocity.0;
		self.camera_at.1 += self.camera_velocity.1;

		self.camera_velocity.0 *= 0.95;
		self.camera_velocity.1 *= 0.95;
		if self.camera_velocity.0.abs() < 0.005 && self.camera_velocity.1.abs() < 0.005 {
			self.camera_velocity = (0.0, 0.0);
		}

		self.scale += self.scale_velocity;
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
		let move_amount = 0.1;

		println!("{event:?}");
		match event {
			Event::MouseEvent(MouseEvent::Scroll { amount, .. }) => {
				self.scale_velocity += amount / 6.0
			}

			Event::KeyboardEvent(_, KeyboardEvent::CharPressed('w')) => {
				self.camera_velocity.1 -= move_amount;
			}
			Event::KeyboardEvent(_, KeyboardEvent::CharPressed('s')) => {
				self.camera_velocity.1 += move_amount;
			}
			Event::KeyboardEvent(_, KeyboardEvent::CharPressed('a')) => {
				self.camera_velocity.0 -= move_amount;
			}
			Event::KeyboardEvent(_, KeyboardEvent::CharPressed('d')) => {
				self.camera_velocity.0 += move_amount;
			}

			Event::KeyboardEvent(_, KeyboardEvent::CharPressed('r')) => {
				*self.tilemap.tiles_mut() = worldgen::gen_tiles();
			}
			_ => {}
		};
		None
	}
}
