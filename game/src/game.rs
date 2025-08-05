use anyhow::{Context, anyhow};
use std::{
	fmt::Debug,
	time::{Duration, Instant},
};
use sui::{
	Compatible, Details, DynamicLayable, Layable, LayableExt,
	core::{Event, KeyboardEvent, MouseEvent, ReturnEvent},
	raylib::ffi::KeyboardKey,
};
use tokio::sync::broadcast;

use crate::{
	comp::{SelectTool, toolbar},
	textures::Textures,
	utils::ReturnEvents,
	world::{
		Tile,
		buildings::BuildingsMap,
		maps::{SIZE, Tilemap, TilemapExt},
		render::TILE_RENDER_SIZE,
		tool::Tool,
	},
};

pub const GAME_TICK_FREQUENCY: Duration = Duration::from_millis(1000 / 20);

/// Singleplayer, self-contained game renderer
#[derive(Debug)]
pub struct Game {
	textures: Textures,

	toolbar: DynamicLayable<'static>,
	tips: Option<DynamicLayable<'static>>,

	pub tilemap: Tilemap,
	pub buildings: BuildingsMap,

	tool: Tool,
	tool_use_tx: broadcast::Sender<((i32, i32), Tool)>,

	/// camera center position in world coordinates
	camera_at: (f32, f32),
	camera_velocity: (f32, f32),
	scale: f32,
	scale_velocity: f32,

	last_game_tick: Instant,
}
impl Game {
	pub fn new(textures: Textures) -> Self {
		// let tilemap = Tilemap::new(SIZE, SIZE); // this causes a multiply overflow in perlin2d for some fucking reason
		let tilemap = Tilemap::stone(SIZE, SIZE);
		let buildings = BuildingsMap::new(SIZE, SIZE);

		Self::from_maps(textures, tilemap, buildings)
	}
	pub fn from_maps(textures: Textures, tilemap: Tilemap, buildings: BuildingsMap) -> Self {
		let (width, height) = tilemap.size();

		let (tool_use_tx, _rx) = broadcast::channel(10);

		Self {
			textures,
			toolbar: Self::gen_toolbar(),
			tips: None,
			tilemap,
			buildings,
			tool: Default::default(),
			tool_use_tx,
			camera_at: (width as f32 / 2.0, height as f32 / 2.0),
			camera_velocity: (0.0, 0.0),
			scale: 1.0,
			scale_velocity: 0.0,
			last_game_tick: Instant::now(),
		}
	}

	pub fn enable_tips<T: Send + Debug + 'static, F: Future<Output = ()> + Send + 'static>(
		&mut self,
		controller: impl FnOnce(
			tokio::sync::mpsc::Sender<stage_manager_remote::RemoteStageChange>,
			tokio::sync::mpsc::Receiver<T>,
		) -> F
		+ Send,
	) {
		let remote = stage_manager_remote::RemoteStage::new(controller);
		let remote = DynamicLayable::new_only_debug(remote);

		self.tips = Some(remote);
	}
	pub fn disable_tips(&mut self) {
		self.tips = None;
	}

	pub fn subscribe_to_tool_use(
		&mut self,
	) -> tokio::sync::broadcast::Receiver<((i32, i32), Tool)> {
		self.tool_use_tx.subscribe()
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
		let comp = self.wrap_as_world(comp, det).overlay(sui::div([
			sui::custom(self.toolbar.immutable_wrap()).into_comp(),
			sui::text(format!("tool: {:?}", self.tool), 24),
		]));

		comp.render(d, det, scale);

		if let Some(tips) = &self.tips {
			let l_det = self.tips_det(det).unwrap();

			tips.render(d, l_det, 1.0);
		}
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

		if let Some(tips) = &mut self.tips {
			tips.tick();
		}
	}

	fn pass_events(
		&mut self,
		events: impl Iterator<Item = Event>,
		det: sui::Details,
		scale: f32,
	) -> impl Iterator<Item = sui::core::ReturnEvent> {
		let mut ret_events = Vec::new();

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
							.pass_events(std::iter::once(Event::MouseEvent(m_event)), l_det, 1.0)
							.for_each(std::mem::drop)
					} else {
						match m_event {
							MouseEvent::Scroll { amount, .. } => {
								self.scale_velocity += amount / 6.0
							}
							MouseEvent::MouseClick { x, y } => {
								let (_, toolbar_h) = self.toolbar.size();

								if y <= toolbar_h {
									match self
										.toolbar
										.pass_events(std::iter::once(event), det, scale)
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

								// use the tool on the block
								// yes the code for that is this large

								// okay something doesn't work idk what
								// toolbar isn't responding to the ReturnEvent so it might be the new SpaceBetween or anything else basically

								let world_pos = || {
									let mut world = self.wrap_as_world(ReturnEvents, det);

									let ret = world.pass_events(std::iter::once(event), det, scale).next().ok_or_else(|| anyhow!(
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
								let world_pos = world_pos().with_context(|| {
									format!(
										"while handling {self:?} use action at screen (x,y) ({x}, {y})"
									)
								});

								let world_pos = match world_pos {
									Ok(a) => a,
									Err(err) => {
										eprintln!("{err}");
										continue;
									}
								};

								{
									let tool = std::mem::take(&mut self.tool);
									tool.r#use(self, world_pos);
									let _ = self.tool_use_tx.send((world_pos, tool.clone()));
									self.tool = tool
								}
							}

							_ => {}
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

				Event::KeyboardEvent(_, KeyboardEvent::CharPressed('r')) => {
					// *self.tilemap.tiles_mut() = worldgen::gen_tiles();
					// TODO reimplement

					let future = async { crate::comp::main().await };
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

		ret_events.into_iter()
	}
}
