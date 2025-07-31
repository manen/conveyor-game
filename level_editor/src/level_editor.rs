use anyhow::{Context, anyhow};
use asset_provider::Assets;
use game::{
	levels::Level,
	textures::Textures,
	utils::ReturnEvents,
	world::{
		ETile,
		render::TILE_RENDER_SIZE,
		tilemap::{SIZE, Tilemap},
	},
};
use nfde::{FilterableDialogBuilder, Nfd, SingleFileDialogBuilder};
use std::{fmt::Debug, path::PathBuf};
use sui::{
	Details, DynamicLayable, Layable, LayableExt,
	core::{Event, KeyboardEvent, MouseEvent},
	raylib::ffi::KeyboardKey,
};

use crate::tools::{self, TileChange};

#[derive(Debug)]
pub struct LevelEditor {
	textures: Textures,
	pub tilemap: Tilemap,

	toolbar: DynamicLayable<'static>,
	placing: ETile,

	/// camera center position in world coordinates
	camera_at: (f32, f32),
	camera_velocity: (f32, f32),
	scale: f32,
	scale_velocity: f32,
}
impl LevelEditor {
	pub fn new<A: Assets + Send + Sync>(
		assets: &A,
		d: &mut sui::Handle,
		thread: &sui::raylib::RaylibThread,
	) -> anyhow::Result<Self> {
		let textures = Textures::new(assets, d, thread)?;
		let tilemap = Tilemap::stone();

		Ok(Self {
			textures,
			tilemap,
			toolbar: DynamicLayable::new(tools::toolbar()),
			placing: ETile::iron_ore(),
			camera_at: (SIZE as f32 / 2.0, SIZE as f32 / 2.0),
			camera_velocity: (0.0, 0.0),
			scale: 1.0,
			scale_velocity: 0.0,
		})
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
impl Layable for LevelEditor {
	fn size(&self) -> (i32, i32) {
		(0, 0)
	}
	fn render(&self, d: &mut sui::Handle, det: sui::Details, scale: f32) {
		self.wrap_as_world(self.tilemap.render(&self.textures), det)
			.overlay(self.toolbar.immutable_wrap())
			.render(d, det, scale);
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
	}

	fn pass_events(
		&mut self,
		events: impl Iterator<Item = Event>,
		det: Details,
		scale: f32,
	) -> impl Iterator<Item = sui::core::ReturnEvent> {
		let move_amount = 0.1;

		let (mut ctrl, mut s) = (false, false);
		for event in events {
			match event {
				Event::MouseEvent(MouseEvent::Scroll { amount, .. }) => {
					self.scale_velocity += amount / 6.0
				}
				Event::MouseEvent(MouseEvent::MouseClick { y, .. }) => {
					let (_, toolbar_h) = self.toolbar.size();

					if y <= toolbar_h {
						match self
							.toolbar
							.pass_events(std::iter::once(event), det, scale)
							.next()
						{
							Some(toolbar_resp) if toolbar_resp.can_take::<TileChange>() => {
								if let Some(TileChange(tool)) = toolbar_resp.take() {
									println!("selected {tool:?}");
									self.placing = tool;
									continue;
								}
							}
							Some(other_event) => {
								println!("non-SelectTool ui return event: {other_event:?}")
							}
							None => {}
						}
					}
				}
				Event::MouseEvent(MouseEvent::MouseHeld { x, y }) => {
					let world_pos = || {
						let mut world = self.wrap_as_world(ReturnEvents, det);

						let ret = world.pass_events(std::iter::once(event), det, scale).next().ok_or_else(|| anyhow!(
								"ReturnEvents didn't actually return an event\nneeded to calculate world position of mouse click"))?;

						let ret: Event = ret.take().ok_or_else(|| {
							anyhow!("ReturnEvents didn't return a sui::core::Event")
						})?;

						match ret {
							Event::MouseEvent(m_event) => {
								let (x, y) = m_event.at();
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
							continue;
						}
					};

					if let Some(target) = self.tilemap.at_mut(world_pos) {
						*target = self.placing.clone();
					}
				}

				Event::KeyboardEvent(_, KeyboardEvent::KeyDown(KeyboardKey::KEY_W)) => {
					self.camera_velocity.1 -= move_amount;
				}
				Event::KeyboardEvent(_, KeyboardEvent::KeyDown(KeyboardKey::KEY_S)) => {
					self.camera_velocity.1 += move_amount;
					s = true;
				}
				Event::KeyboardEvent(_, KeyboardEvent::KeyDown(KeyboardKey::KEY_A)) => {
					self.camera_velocity.0 -= move_amount;
				}
				Event::KeyboardEvent(_, KeyboardEvent::KeyDown(KeyboardKey::KEY_D)) => {
					self.camera_velocity.0 += move_amount;
				}

				Event::KeyboardEvent(_, KeyboardEvent::KeyDown(KeyboardKey::KEY_LEFT_CONTROL)) => {
					ctrl = true;
				}

				_ => {}
			}
		}

		if ctrl && s {
			let level = Level::from_tilemap(&self.tilemap);

			std::thread::spawn(|| {
				let nfd = Nfd::new().map_err(|err| anyhow!("nfde error: {err}"))?;

				let mut res = nfd.save_file();
				let res = res
					.add_filter("level files", "cglf")
					.map_err(|err| anyhow!("error while creating save dialog: {err}"))?;
				let res = tokio::task::block_in_place(move || res.show());

				use nfde::DialogResult;
				match res {
					DialogResult::Ok(path) => {
						let path = PathBuf::from(path.as_path());
						let a = async move { level.save(path).await };
						tokio::spawn(a);
					}
					DialogResult::Cancel => return Ok(()),
					DialogResult::Err(error_str) => return Err(anyhow!(error_str)),
				};

				anyhow::Ok(())
			});
		}

		std::iter::empty()
	}
}

#[derive(Debug)]
pub enum SaveState {
	NotSaving,
	Dialog(std::thread::JoinHandle<nfde::DialogResult<nfde::NfdPathBuf>>),
	Save(tokio::task::JoinHandle<anyhow::Result<()>>),
}
