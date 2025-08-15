use anyhow::{Context, anyhow};
use stage_manager_remote::{RemoteStage, RemoteStageChange};
use std::{
	fmt::Debug,
	time::{Duration, Instant},
};
use sui::{
	Compatible, Details, DynamicLayable, Layable, LayableExt,
	core::{Event, KeyboardEvent, MouseEvent, ReturnEvent},
	raylib::ffi::KeyboardKey,
};
use tokio::sync::{broadcast, mpsc};

use crate::{
	comp::{SelectTool, toolbar},
	textures::Textures,
	utils::ReturnEvents,
	world::{
		EResource, Resource,
		buildings::BuildingsMap,
		maps::{SIZE, Tilemap, TilemapExt},
		render::TILE_RENDER_SIZE,
	},
};

mod tool;
pub use tool::*;
mod data;
pub use data::*;
pub mod timer;
pub use timer::Timer;
use timer::TimerRenderable;
mod runner;
pub use runner::*;
pub mod goal;
pub use goal::Goal;

pub const GAME_TICK_FREQUENCY: Duration = Duration::from_millis(1000 / 20);

/// Singleplayer, self-contained game renderer
#[derive(Debug)]
pub struct Game {
	textures: Textures,
	data: GameData,

	toolbar: DynamicLayable<'static>,
	tips: Option<DynamicLayable<'static>>,

	tool: Tool,
	tool_use_tx: broadcast::Sender<((i32, i32), Tool)>,

	timer: Option<Timer>,
	paused: bool,
	can_toggle_time: bool,

	goal_display: Option<DynamicLayable<'static>>,

	/// camera center position in world coordinates
	camera_at: (f32, f32),
	camera_velocity: (f32, f32),
	scale: f32,
	scale_velocity: f32,

	last_tick: Instant,
	last_game_tick: Instant,
}
impl Game {
	pub fn new(textures: Textures) -> Self {
		let tilemap = Tilemap::new(SIZE, SIZE); // this causes a multiply overflow in perlin2d for some fucking reason
		// let tilemap = Tilemap::stone(SIZE, SIZE);
		let buildings = BuildingsMap::new(SIZE, SIZE);

		Self::from_maps(textures, tilemap, buildings)
	}
	pub fn from_maps(textures: Textures, tilemap: Tilemap, buildings: BuildingsMap) -> Self {
		let (width, height) = tilemap.size();
		let data = GameData::new(tilemap, buildings);

		let (tool_use_tx, _rx) = broadcast::channel(10);

		Self {
			toolbar: sui::custom(toolbar(&textures)),
			textures,
			tips: None,
			timer: None,
			goal_display: None,
			paused: false,
			can_toggle_time: true,
			data,
			tool: Default::default(),
			tool_use_tx,
			camera_at: (width as f32 / 2.0, height as f32 / 2.0),
			camera_velocity: (0.0, 0.0),
			scale: 1.0,
			scale_velocity: 0.0,
			last_tick: Instant::now(),
			last_game_tick: Instant::now(),
		}
	}

	pub fn enable_tips_spawn<T: Send + Debug + 'static, F: Future<Output = ()> + Send + 'static>(
		&mut self,
		controller: impl FnOnce(
			tokio::sync::mpsc::Sender<stage_manager_remote::RemoteStageChange>,
			tokio::sync::mpsc::Receiver<T>,
		) -> F
		+ Send,
	) {
		let remote = stage_manager_remote::RemoteStage::spawn_new(controller);
		let remote = DynamicLayable::new_only_debug(remote);

		self.tips = Some(remote);
	}
	pub fn enable_tips<T: Send + Debug + 'static>(
		&mut self,
	) -> (mpsc::Sender<RemoteStageChange>, mpsc::Receiver<T>) {
		let ((stage_tx, events_rx), stage) = RemoteStage::new();
		self.tips = Some(sui::custom_only_debug(stage));

		(stage_tx, events_rx)
	}
	pub fn disable_tips(&mut self) {
		self.tips = None;
	}

	pub fn enable_goal_display_spawn<
		T: Send + Debug + 'static,
		F: Future<Output = ()> + Send + 'static,
	>(
		&mut self,
		controller: impl FnOnce(
			tokio::sync::mpsc::Sender<stage_manager_remote::RemoteStageChange>,
			tokio::sync::mpsc::Receiver<T>,
		) -> F
		+ Send,
	) {
		let remote = stage_manager_remote::RemoteStage::spawn_new(controller);
		let remote = DynamicLayable::new_only_debug(remote);

		self.tips = Some(remote);
	}
	pub fn enable_goal_display<T: Send + Debug + 'static>(
		&mut self,
	) -> (mpsc::Sender<RemoteStageChange>, mpsc::Receiver<T>) {
		let ((stage_tx, events_rx), stage) =
			// RemoteStage::new_explicit(sui::comp::Color::new(sui::Color::PURPLE).fix_wh_square(400));
			RemoteStage::new();
		self.goal_display = Some(sui::custom_only_debug(stage));

		(stage_tx, events_rx)
	}
	pub fn disable_goal_display(&mut self) {
		self.goal_display = None;
	}

	pub fn subscribe_to_tool_use(
		&mut self,
	) -> tokio::sync::broadcast::Receiver<((i32, i32), Tool)> {
		self.tool_use_tx.subscribe()
	}

	/// sets and starts the timer if the game is started; use self.pause_time() to spawn in paused
	pub fn enable_timer(&mut self, target: Duration) {
		self.timer = Some(Timer::new(target));

		if self.paused {
			self.pause_time();
		} else {
			self.resume_time();
		}
	}
	pub fn disable_timer(&mut self) {
		self.timer = None;
	}

	/// sets whether the user can toggle the passage of time with the spacebar
	pub fn set_can_toggle_time(&mut self, can_switch_time: bool) {
		self.can_toggle_time = can_switch_time;
	}

	pub fn pause_time(&mut self) {
		self.paused = true;
		if let Some(timer) = &mut self.timer {
			timer.pause();
		}
	}
	pub fn resume_time(&mut self) {
		self.paused = false;
		if let Some(timer) = &mut self.timer {
			timer.resume();
		}
	}
	pub fn toggle_time(&mut self) {
		if self.paused {
			self.resume_time();
		} else {
			self.pause_time();
		}
	}
	pub fn is_paused(&self) -> bool {
		self.paused
	}

	pub fn tile_resource_at(&self, pos: (i32, i32)) -> Option<EResource> {
		self.data.tile_resource_at(pos)
	}

	pub fn data(&self) -> &GameData {
		&self.data
	}
	pub fn buildings(&self) -> &BuildingsMap {
		&self.data.buildings
	}
	pub fn tiles(&self) -> &Tilemap {
		&self.data.tilemap
	}

	pub fn tips_det(&self, det: Details) -> Option<Details> {
		if let Some(tips) = &self.tips {
			let (_, h) = tips.size();
			let l_det = Details {
				x: 0,
				y: det.ah - h,
				aw: det.aw,
				ah: h,
			};
			Some(l_det)
		} else {
			None
		}
	}
	pub fn goal_display_det(&self, det: Details) -> Option<Details> {
		if let Some(goal_display) = &self.goal_display {
			let (w, _) = goal_display.size();
			// on the right side of the screen, between the toolbar and the tips

			let start_x = det.aw - w;
			let start_y = self.toolbar.size().1;

			let aw = w;
			let ah = self
				.tips_det(det)
				.map(|tips_det| tips_det.y - start_y)
				.unwrap_or_else(|| det.ah - start_y);

			let l_det = Details {
				x: start_x,
				y: start_y,
				aw,
				ah,
			};
			Some(l_det)
		} else {
			None
		}
	}

	fn gen_toolbar(&self) -> DynamicLayable<'static> {
		DynamicLayable::new(toolbar(&self.textures))
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
		let stage_comp = self
			.data
			.tilemap
			.render(&self.textures)
			.overlay(self.data.buildings.render(&self.textures));
		let comp = self.wrap_as_world(stage_comp, det);

		let timer = if let Some(timer) = &self.timer {
			let render = sui::custom_only_debug(timer.render()); // timer.render() is already centered
			render.into_comp()
		} else {
			sui::Comp::Space(sui::comp::Space::new(0, 0))
		};
		let ui = sui::div([
			sui::custom(self.toolbar.immutable_wrap()).into_comp(),
			sui::Text::new(format!("tool: {:?}", self.tool), 24).into_comp(),
			timer,
		]);
		let comp = comp.overlay(ui);

		comp.render(d, det, scale);

		if let Some(tips) = &self.tips {
			let l_det = self.tips_det(det).unwrap();

			tips.render(d, l_det, 1.0);
		}
		if let Some(goal_display) = &self.goal_display {
			let l_det = self.goal_display_det(det).unwrap();

			goal_display.render(d, l_det, 1.0);
		}
	}

	fn tick(&mut self) {
		let delta = self.last_tick.elapsed().as_secs_f32();

		let tile_render_size = TILE_RENDER_SIZE as f32 * self.real_scale();

		// world coords
		// move amounts are calculated based on the zoom, the point is that every move will move the same number of pixels
		// no matter the scale...
		let move_amount_x = 1.0 / tile_render_size * TILE_RENDER_SIZE as f32;
		let move_amount_y = 1.0 / tile_render_size * TILE_RENDER_SIZE as f32;

		let move_amount_x = self.camera_velocity.0 * 0.85 * move_amount_x;
		let move_amount_y = self.camera_velocity.1 * 0.85 * move_amount_y;

		let move_amount_x = move_amount_x * 60.0 * delta;
		let move_amount_y = move_amount_y * 60.0 * delta;

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

		let scale_taken = self.scale_velocity * delta * 120.0;
		self.scale += scale_taken;
		self.scale = self.scale.max(-40.0).min(60.0);

		let scale_reduce = self.scale_velocity * 18.0 * delta;
		self.scale_velocity = self.scale_velocity - scale_reduce;
		if self.scale_velocity.abs() < 0.05 {
			self.scale_velocity = 0.0;
		}

		if !self.paused {
			if self.last_game_tick.elapsed() >= GAME_TICK_FREQUENCY {
				self.data.tick();
				self.last_game_tick = Instant::now();
			}
			if let Some(timer) = &mut self.timer {
				timer.tick();
			}
		}

		if let Some(tips) = &mut self.tips {
			tips.tick();
		}
		if let Some(goal_display) = &mut self.goal_display {
			goal_display.tick();
		}
		self.last_tick = Instant::now();
	}

	fn pass_events(
		&mut self,
		events: impl Iterator<Item = Event>,
		det: sui::Details,
		scale: f32,
		ret_events: &mut Vec<ReturnEvent>,
	) {
		macro_rules! world_pos {
			($m_event:expr, $err_msg:expr) => {{
				let world_pos = || {
					let mut world = self.wrap_as_world(ReturnEvents, det);

					let (x, y) = $m_event.at();
					let ret = world.pass_events_simple(std::iter::once(Event::MouseEvent(MouseEvent::MouseClick { x, y })), det, scale).into_iter().next().ok_or_else(|| anyhow!(
				"ReturnEvents didn't actually return an event\nneeded to calculate world position of mouse click"))?;

					let ret: Event = ret.take().ok_or_else(|| {
						anyhow!("ReturnEvents didn't return a sui::core::Event")
					})?;

					match ret {
						Event::MouseEvent(MouseEvent::MouseClick { x, y }) => {
							Ok((x / TILE_RENDER_SIZE, y / TILE_RENDER_SIZE))
						}
						_ => Err(anyhow!(
							"expected MouseEvent::MouseClick, got {ret:?}"
						)),
					}
				};
				let world_pos = world_pos().with_context(|| format!($err_msg));

				world_pos
			}};
		}

		let move_amount = 0.1;
		for event in events {
			match event {
				Event::MouseEvent(m_event) => {
					let (mouse_x, mouse_y) = m_event.at();

					let mut pass_to_tips = false;
					if let Some(tips) = &self.tips {
						let (_, h) = tips.size();

						let from_y = det.ah - h;
						if mouse_y >= from_y {
							pass_to_tips = true;
						}
					}

					if pass_to_tips {
						let l_det = self.tips_det(det).unwrap();

						self.tips
							.as_mut()
							.unwrap()
							.pass_events_simple(
								std::iter::once(Event::MouseEvent(m_event)),
								l_det,
								1.0,
							)
							.into_iter()
							.for_each(std::mem::drop)
					} else {
						match m_event {
							MouseEvent::Scroll { amount, .. } => {
								self.scale_velocity += amount / 2.0
							}
							MouseEvent::MouseClick { x, y } => {
								let (_, toolbar_h) = self.toolbar.size();

								if y <= toolbar_h {
									match self
										.toolbar
										.pass_events_simple(std::iter::once(event), det, scale)
										.into_iter()
										.next()
									{
										Some(toolbar_resp)
											if toolbar_resp.can_take::<SelectTool>() =>
										{
											if let Some(SelectTool(tool)) = toolbar_resp.take() {
												println!("selected {tool:?}");
												self.tool = tool;
												continue;
											}
										}
										Some(other_event) => {
											println!(
												"non-SelectTool ui return event: {other_event:?}"
											)
										}
										None => {}
									}
								}

								let world_pos = match world_pos!(
									m_event,
									"couldn't get world_pos in MouseClick"
								) {
									Ok(a) => a,
									Err(err) => {
										eprintln!("{err}");
										continue;
									}
								};

								self.tool.r#use(&mut self.data, world_pos);
								let _ = self.tool_use_tx.send((world_pos, self.tool.clone()));
							}
							MouseEvent::MouseHeld { .. } => {
								let world_pos = match world_pos!(
									m_event,
									"couldn't get world_pos in MouseHeld"
								) {
									Ok(a) => a,
									Err(err) => {
										eprintln!("{err}");
										continue;
									}
								};

								self.tool.held(&mut self.data, world_pos);
							}
							MouseEvent::MouseRelease { .. } => {
								let world_pos = match world_pos!(
									m_event,
									"couldn't get world_pos in MouseRelease"
								) {
									Ok(a) => a,
									Err(err) => {
										eprintln!("{err}");
										continue;
									}
								};

								self.tool.release(&mut self.data, world_pos);
							}
						}
					}
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
				Event::KeyboardEvent(_, KeyboardEvent::CharPressed(' ')) => {
					if self.can_toggle_time {
						self.toggle_time();
					}
				}

				Event::KeyboardEvent(_, KeyboardEvent::CharPressed('r')) => {
					// *self.tilemap.tiles_mut() = worldgen::gen_tiles();
					// TODO reimplement

					let future = async { crate::scripts::main::main().await };
					let loader = stage_manager_loaders::Loader::new_overlay(
						sui::comp::Space::new(10, 10),
						future,
						|a| stage_manager::StageChange::Simple(a),
					);
					ret_events.push(ReturnEvent::new(loader));
				}

				_ => {
					// println!("{event:?}")
				}
			};
		}
	}
}
