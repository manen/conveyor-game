// use std::time::{Duration, Instant};

use sui::{Layable, LayableExt, raylib::prelude::RaylibDraw, tex::Texture};

use crate::{
	textures::{TextureID, Textures},
	utils::Direction,
	world::{EResource, Resource, buildings::Building, render::TILE_RENDER_SIZE},
};

pub const CONVEYOR_CAPACITY: usize = 3;
// pub const CONVEYOR_SPEED: Duration = Duration::from_millis(200);

#[derive(Clone, Debug)]
pub struct Conveyor {
	pub dir: Direction,

	holding: heapless::Deque<EResource, CONVEYOR_CAPACITY>,
	// last_tick: Instant,
}
impl Conveyor {
	pub fn new(dir: Direction) -> Self {
		Self {
			dir,
			holding: heapless::Deque::default(),
			// last_tick: Instant::now(),
		}
	}
}

impl Building for Conveyor {
	fn name(&self) -> std::borrow::Cow<'static, str> {
		"conveyor".into()
	}
	fn texture_id(&self) -> TextureID {
		TextureID::ConveyorTop
	}

	fn render<'a>(
		&'a self,
		textures: &'a Textures,
	) -> impl sui::Layable + Clone + std::fmt::Debug + 'a {
		#[derive(Clone, Debug)]
		struct ConveyorRenderer<'a> {
			textures: &'a Textures,
			dir: Direction,
			holding: &'a heapless::Deque<EResource, CONVEYOR_CAPACITY>,
		}
		impl<'a> Layable for ConveyorRenderer<'a> {
			fn size(&self) -> (i32, i32) {
				(TILE_RENDER_SIZE, TILE_RENDER_SIZE)
			}
			/// det.aw is trusted to be TILE_RENDER_SIZE scaled properly
			fn render(&self, d: &mut sui::Handle, det: sui::Details, _scale: f32) {
				let top_texture = self.textures.texture_for(TextureID::ConveyorTop);
				if let Some(top_texture) = top_texture {
					// draws the top facing texture rotated the right way

					let (x_offset, y_offset) = match self.dir {
						Direction::Top => (0, 0),
						Direction::Right => (1, 0),
						Direction::Bottom => (1, 1),
						Direction::Left => (0, 1),
					};
					let (x_offset, y_offset) = (x_offset * det.aw, y_offset * det.aw);

					let tex_det = sui::Details {
						x: det.x + x_offset,
						y: det.y + y_offset,
						..det
					};
					top_texture.render_with_rotation(d, tex_det, self.dir.degrees());
				} else {
					d.draw_rectangle(det.x, det.y, det.aw, det.ah, sui::Color::PURPLE);
				}

				// holding
				{
					let holding_textures = self
						.holding
						.iter()
						.map(|a| self.textures.texture_for(a.texture_id()));

					let new = if self.dir.is_axis_same(&Direction::Right) {
						sui::comp::div::SpaceBetween::new_horizontal
					} else {
						sui::comp::div::SpaceBetween::new
					};
					let mut content = holding_textures
						.map(|tex| {
							match tex {
								Some(tex) => sui::custom(tex.immutable_wrap()),
								None => sui::custom(sui::comp::Space::new(0, 0)),
							}
							.fix_wh_square(det.aw / 2)
						})
						.collect::<Vec<_>>();

					let should_reverse = match self.dir {
						Direction::Top | Direction::Left => false,
						Direction::Bottom | Direction::Right => true,
					};
					if should_reverse {
						content.reverse();
					}

					let holding_full = new(content).centered().margin(4);
					holding_full.render(d, det, 1.0);
				}
			}
		}

		ConveyorRenderer {
			textures,
			dir: self.dir,
			holding: &self.holding,
		}
	}
	fn tool_icon_render(
		&self,
		textures: &Textures,
	) -> impl Layable + Clone + std::fmt::Debug + 'static {
		#[derive(Clone, Debug)]
		struct ConveyorIconRenderer {
			texture: Option<Texture>,
			dir: Direction,
		}
		impl Layable for ConveyorIconRenderer {
			fn size(&self) -> (i32, i32) {
				(64, 64)
			}
			fn render(&self, d: &mut sui::Handle, det: sui::Details, scale: f32) {
				let det = det.mul_size(scale);

				if let Some(top_texture) = &self.texture {
					// draws the top facing texture rotated the right way

					let (x_offset, y_offset) = match self.dir {
						Direction::Top => (0, 0),
						Direction::Right => (1, 0),
						Direction::Bottom => (1, 1),
						Direction::Left => (0, 1),
					};
					let (x_offset, y_offset) = (x_offset * det.aw, y_offset * det.aw);

					let tex_det = sui::Details {
						x: det.x + x_offset,
						y: det.y + y_offset,
						..det
					};
					top_texture.render_with_rotation(d, tex_det, self.dir.degrees());
				} else {
					d.draw_rectangle(det.x, det.y, det.aw, det.ah, sui::Color::PURPLE);
				}
			}
		}

		ConveyorIconRenderer {
			dir: self.dir,
			texture: textures.texture_for(self.texture_id()).cloned(),
		}
	}

	fn can_receive(&self, _from: Option<Direction>) -> bool {
		CONVEYOR_CAPACITY as i32 - self.holding.len() as i32 > 0
	}
	fn capacity_for(&self, _resource: &EResource, _from: Option<Direction>) -> i32 {
		CONVEYOR_CAPACITY as i32 - self.holding.len() as i32
	}
	fn receive(&mut self, resource: EResource, _from: Option<Direction>) {
		if self.capacity_for(&resource, _from) > 0 {
			let _ = self.holding.push_back(resource);
			// self.last_tick = Instant::now();
		}
	}

	fn needs_poll(&self) -> bool {
		!self.holding.is_empty()
		//  && self.last_tick.elapsed() > CONVEYOR_SPEED
	}
	fn resource_sample(
		&self,
		_tile_resource: Option<EResource>,
		_to: Option<Direction>,
	) -> Option<EResource> {
		self.holding.iter().next().cloned()
	}
	fn poll_resource(
		&mut self,
		_tile_resource: Option<EResource>,
		_to: Option<Direction>,
	) -> Option<EResource> {
		self.holding.pop_front()
	}

	fn pass_relatives(&self) -> &'static [(i32, i32)] {
		self.dir.rel_array()
	}
	fn rank_pass_source(&self, relative_pos: (i32, i32)) -> i32 {
		let from_dir = match Direction::from_rel(relative_pos) {
			Some(a) => a,
			None => return 0,
		};
		match (self.dir, from_dir) {
			(a, b) if a == b => 15,
			(a, b) if a.reverse() == b => 10, // same direction
			_ => 5,
		}
	}
}
