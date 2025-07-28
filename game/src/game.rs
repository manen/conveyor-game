use anyhow::{Context, anyhow};
use asset_provider::Assets;
use std::{
	fmt::Debug,
	time::{Duration, Instant},
};
use sui::{
	Compatible, Details, DynamicLayable, Layable, LayableExt,
	core::{Event, KeyboardEvent, MouseEvent},
	raylib::ffi::KeyboardKey,
};

use crate::{
	comp::{SelectTool, toolbar},
	textures::Textures,
	utils::ReturnEvents,
	world::{
		Tile,
		buildings::BuildingsMap,
		render::TILE_RENDER_SIZE,
		tilemap::{SIZE, Tilemap},
		tool::Tool,
		worldgen,
	},
};

pub const GAME_TICK_FREQUENCY: Duration = Duration::from_millis(1000 / 20);

/// Singleplayer, self-contained game renderer
#[derive(Debug)]
pub struct Game {
	textures: Textures,
	toolbar: DynamicLayable<'static>,

	pub tilemap: Tilemap,
	pub buildings: BuildingsMap,

	tool: Tool,

	/// camera center position in world coordinates
	camera_at: (f32, f32),
	camera_velocity: (f32, f32),
	scale: f32,
	scale_velocity: f32,

	last_game_tick: Instant,
}
impl Game {
	pub fn new<A: Assets + Send + Sync>(
		assets: &A,
		d: &mut sui::Handle,
		thread: &sui::raylib::RaylibThread,
	) -> anyhow::Result<Self> {
		let textures = Textures::new(assets, d, thread)?;
		let tilemap = Tilemap::new();
		let buildings = BuildingsMap::default();

		Ok(Self {
			textures,
			toolbar: Self::gen_toolbar(),
			tilemap,
			buildings,
			tool: Default::default(),
			camera_at: (SIZE as f32 / 2.0, SIZE as f32 / 2.0),
			camera_velocity: (0.0, 0.0),
			scale: 1.0,
			scale_velocity: 0.0,
			last_game_tick: Instant::now(),
		})
	}

	fn gen_toolbar() -> DynamicLayable<'static> {
		DynamicLayable::new(toolbar())
	}
	fn wrap_as_world<L: Layable + Debug + Clone>(
		&self,
		layable: L,
		det: Details,
	) -> impl Layable + Debug + Clone {
		let real_scale = self.real_scale();
		layable.scale(real_scale).view(
			(self.camera_at.0 * TILE_RENDER_SIZE as f32 * real_scale) as i32 - det.aw / 2,
			(self.camera_at.1 * TILE_RENDER_SIZE as f32 * real_scale) as i32 - det.ah / 2,
		)
	}

	fn real_scale(&self) -> f32 {
		(1.1 as f32).powf(self.scale)
	}
}

impl Layable for Game {
	fn size(&self) -> (i32, i32) {
		let size = TILE_RENDER_SIZE * SIZE as i32;
		(size, size)
	}

	/// we ignore scale
	fn render(&self, d: &mut sui::Handle, det: sui::Details, scale: f32) {
		let comp = self
			.tilemap
			.render(&self.textures)
			.overlay(self.buildings.render(&self.textures));
		let comp = self
			.wrap_as_world(comp, det)
			.overlay(sui::div([self.toolbar.immutable_wrap()]));

		comp.render(d, det, scale);
	}

	fn tick(&mut self) {
		let tile_render_size = TILE_RENDER_SIZE as f32 * self.real_scale();

		// world coords
		// move amounts are calculated based on the zoom, the point is that every move will move the same number of pixels
		// no matter the scale...
		let move_amount_x = 1.0 / tile_render_size * TILE_RENDER_SIZE as f32;
		let move_amount_y = 1.0 / tile_render_size * TILE_RENDER_SIZE as f32;

		let move_amount_x = self.camera_velocity.0 * 0.85 * move_amount_x;
		let move_amount_y = self.camera_velocity.1 * 0.85 * move_amount_y;

		// ...except if it'd move too many tiles away
		let move_limit = 0.2;
		let (move_amount_x, move_amount_y) = (
			move_amount_x.min(move_limit).max(-move_limit),
			move_amount_y.min(move_limit).max(-move_limit),
		);

		self.camera_at.0 += move_amount_x;
		self.camera_at.1 += move_amount_y;

		self.camera_velocity.0 *= 0.85;
		self.camera_velocity.1 *= 0.85;
		if self.camera_velocity.0.abs() < 0.005 && self.camera_velocity.1.abs() < 0.005 {
			self.camera_velocity = (0.0, 0.0);
		}

		self.scale += self.scale_velocity;
		self.scale = self.scale.max(-40.0).min(60.0);

		self.scale_velocity *= 0.95;
		if self.scale_velocity.abs() < 0.005 {
			self.scale_velocity = 0.0;
		}

		if self.last_game_tick.elapsed() >= GAME_TICK_FREQUENCY {
			let tile_resource_at = |pos| {
				let tile = self.tilemap.at(pos)?;
				let resource = tile.generate_resource();
				resource
			};
			self.buildings.tick(tile_resource_at);

			self.last_game_tick = Instant::now();
		}
	}

	fn pass_event(
		&mut self,
		event: Event,
		det: sui::Details,
		scale: f32,
	) -> Option<sui::core::ReturnEvent> {
		let move_amount = 0.1;

		match event {
			Event::MouseEvent(MouseEvent::Scroll { amount, .. }) => {
				self.scale_velocity += amount / 6.0
			}
			Event::MouseEvent(MouseEvent::MouseClick { x, y }) => {
				let (_, toolbar_h) = self.toolbar.size();

				if y <= toolbar_h {
					match self.toolbar.pass_event(event, det, scale) {
						Some(toolbar_resp) if toolbar_resp.can_take::<SelectTool>() => {
							if let Some(SelectTool(tool)) = toolbar_resp.take() {
								println!("selected {tool:?}");
								self.tool = tool;
								return None;
							}
						}
						Some(other_event) => {
							println!("non-SelectTool ui return event: {other_event:?}")
						}
						None => {}
					}
				}

				// use the tool on the block
				// yes the code for that is this large

				// okay something doesn't work idk what
				// toolbar isn't responding to the ReturnEvent so it might be the new SpaceBetween or anything else basically

				let world_pos = || {
					let mut world = self.wrap_as_world(ReturnEvents, det);

					let ret = world.pass_event(event, det, scale).ok_or_else(|| anyhow!(
								"ReturnEvents didn't actually return an event\nneeded to calculate world position of mouse click"))?;

					let ret: Event = ret
						.take()
						.ok_or_else(|| anyhow!("ReturnEvents didn't return a sui::core::Event"))?;

					match ret {
						Event::MouseEvent(MouseEvent::MouseClick { x, y }) => {
							Ok((x / TILE_RENDER_SIZE, y / TILE_RENDER_SIZE))
						}
						_ => Err(anyhow!("expected MouseEvent::MouseClick, got {ret:?}")),
					}
				};
				let world_pos = world_pos().with_context(|| {
					format!("while handling {self:?} use action at screen (x,y) ({x}, {y})")
				});

				let world_pos = match world_pos {
					Ok(a) => a,
					Err(err) => {
						eprintln!("{err}");
						return None;
					}
				};

				let tool = std::mem::take(&mut self.tool);
				tool.r#use(self, world_pos);
				self.tool = tool
			}

			Event::KeyboardEvent(_, KeyboardEvent::KeyDown(KeyboardKey::KEY_W)) => {
				self.camera_velocity.1 -= move_amount;
			}
			Event::KeyboardEvent(_, KeyboardEvent::KeyDown(KeyboardKey::KEY_S)) => {
				self.camera_velocity.1 += move_amount;
			}
			Event::KeyboardEvent(_, KeyboardEvent::KeyDown(KeyboardKey::KEY_A)) => {
				self.camera_velocity.0 -= move_amount;
			}
			Event::KeyboardEvent(_, KeyboardEvent::KeyDown(KeyboardKey::KEY_D)) => {
				self.camera_velocity.0 += move_amount;
			}

			Event::KeyboardEvent(_, KeyboardEvent::CharPressed('r')) => {
				*self.tilemap.tiles_mut() = worldgen::gen_tiles();
			}
			Event::KeyboardEvent(_, KeyboardEvent::CharPressed('t')) => {
				self.tool.cycle();
			}

			_ => {
				// println!("{event:?}")
			}
		};
		None
	}
}
