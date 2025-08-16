use anyhow::{Context, anyhow};
use arc_swap::ArcSwap;
use game::{
	levels::Level,
	textures::Textures,
	utils::ReturnEvents,
	world::{
		ETile,
		maps::{Tilemap, TilemapExt},
		render::TILE_RENDER_SIZE,
	},
};
use rfd::AsyncFileDialog;
use std::{
	fmt::Debug,
	hash::{DefaultHasher, Hash, Hasher},
	ops::Deref,
	path::PathBuf,
	sync::Arc,
};
use sui::{
	Details, DynamicLayable, Layable, LayableExt,
	core::{Event, KeyboardEvent, MouseEvent, ReturnEvent},
	raylib::ffi::KeyboardKey,
};

use crate::tools::{self, TileChange};

#[derive(Debug)]
pub struct LevelEditor {
	textures: Textures,
	pub tilemap: Tilemap,

	saving_handle: Option<tokio::task::JoinHandle<anyhow::Result<()>>>,
	last_save_hash: Arc<ArcSwap<u64>>,

	toolbar: DynamicLayable<'static>,
	placing: ETile,

	/// camera center position in world coordinates
	camera_at: (f32, f32),
	camera_velocity: (f32, f32),
	scale: f32,
	scale_velocity: f32,
}
impl LevelEditor {
	pub fn new(width: usize, height: usize, textures: Textures) -> Self {
		let tilemap = Tilemap::stone(width, height);
		Self::from_tilemap(tilemap, textures)
	}
	pub fn from_tilemap(tilemap: Tilemap, textures: Textures) -> Self {
		let (width, height) = tilemap.size();

		let level_editor = Self {
			textures,
			tilemap,
			saving_handle: None,
			last_save_hash: Arc::new(ArcSwap::from_pointee(0)),
			toolbar: DynamicLayable::new(tools::toolbar()),
			placing: ETile::stone(),
			camera_at: (width as f32 / 2.0, height as f32 / 2.0),
			camera_velocity: (0.0, 0.0),
			scale: 1.0,
			scale_velocity: 0.0,
		};
		level_editor.hash_tiles();
		level_editor
	}

	fn hash_get(&self) -> u64 {
		let mut hasher = DefaultHasher::default();
		self.tilemap.hash(&mut hasher);
		hasher.finish()
	}
	/// hashes the tiles and saves them into self.last_save_hash \
	/// so essentially make it so the game thinks you just saved
	fn hash_tiles(&self) {
		self.last_save_hash.store(Arc::new(self.hash_get()));
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

		if let Some(handle) = &self.saving_handle {
			if handle.is_finished() {
				println!("saving finished");
				self.saving_handle = None;
			}
		}
	}

	fn pass_events(
		&mut self,
		events: impl Iterator<Item = Event>,
		det: Details,
		scale: f32,
		ret_events: &mut Vec<ReturnEvent>,
	) {
		let move_amount = 0.1;

		let (mut ctrl, mut s, mut esc) = (false, false, false);
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
							.pass_events_simple(std::iter::once(event), det, scale)
							.into_iter()
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

						let ret = world.pass_events_simple(std::iter::once(event), det, scale).into_iter().next().ok_or_else(|| anyhow!(
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

				Event::KeyboardEvent(_, KeyboardEvent::KeyDown(KeyboardKey::KEY_ESCAPE)) => {
					esc = true;
				}

				_ => {}
			}
		}

		if esc {
			let saved = *self.last_save_hash.load().deref().deref() == self.hash_get();
			match (saved, ctrl) {
				(true, _) | (_, true) => {
					let stage_change =
						stage_manager::StageChange::simple_only_debug(crate::creation_screen());
					ret_events.push(ReturnEvent::new(stage_change));
				}
				(false, false) => {
					println!(
						"you have unsaved progress. save with ctrl + s, or discard and quit with ctrl + esc"
					)
				}
			}
		}

		if ctrl && s {
			if self
				.saving_handle
				.as_ref()
				.map(|h| h.is_finished())
				.unwrap_or(true)
			{
				let level = Level::from_tilemap(&self.tilemap);
				let current_hash = self.hash_get();
				let save_hash = self.last_save_hash.clone();

				let handle = tokio::spawn(async move {
					let files = AsyncFileDialog::new()
						.add_filter("level file", &["cglf"])
						.set_directory(std::env::current_dir()?)
						.set_title("saving level")
						.set_file_name("new-level.cglf")
						.save_file()
						.await;

					if let Some(files) = files {
						let path = PathBuf::from(files.path());
						println!("saving to {path:?}");
						level.save(path).await?;

						save_hash.swap(Arc::new(current_hash));
					} else {
						eprintln!("file saving dialog didn't return a file handle");
					}
					Ok(())
				});

				self.saving_handle = Some(handle);
			}
		}
	}
}
