// pub mod small_extractor;
// pub use small_extractor::*;

use std::{borrow::Cow, collections::VecDeque, fmt::Debug};

use sui::{Layable, raylib::prelude::RaylibDraw};

use crate::{
	textures::{TextureID, Textures},
	utils::Direction,
	world::{
		EResource,
		render::{self, TILE_RENDER_SIZE},
		tilemap::SIZE,
	},
};

mod small_extractor;
pub use small_extractor::*;
mod debug_consumer;
pub use debug_consumer::*;
mod conveyor;
pub use conveyor::*;

pub trait Building {
	fn name(&self) -> Cow<'static, str>;
	fn texture_id(&self) -> TextureID;

	fn render<'a>(&'a self, textures: &'a Textures) -> impl Layable + Clone + Debug + 'a {
		#[derive(Clone, Debug)]
		struct DefaultBuildingRender<'a> {
			texture_id: TextureID,
			textures: &'a Textures,
		}
		impl<'a> Layable for DefaultBuildingRender<'a> {
			fn size(&self) -> (i32, i32) {
				(TILE_RENDER_SIZE, TILE_RENDER_SIZE)
			}
			/// scale is ignored; send properly sized det
			fn render(&self, d: &mut sui::Handle, det: sui::Details, _scale: f32) {
				let tex = self.textures.texture_for_b(&self.texture_id);
				match tex {
					None => {
						d.draw_rectangle(det.x, det.y, det.aw, det.ah, sui::Color::PURPLE);
					}
					Some(tex) => {
						tex.render(d, det);
					}
				}
			}
		}

		DefaultBuildingRender {
			texture_id: self.texture_id(),
			textures,
		}
	}

	// receiving is how shit gets passed
	fn can_receive(&self, _resource: &EResource) -> bool {
		false
	}
	fn receive(&mut self, _resource: EResource) {}

	/// [Self::poll_resource], without advancing any internal timers or anything
	fn resource_sample(&self, _tile_resource: Option<EResource>) -> Option<EResource> {
		None
	}
	// polling is how you generate new shit
	fn needs_poll(&self) -> bool {
		false
	}
	fn poll_resource(&mut self, _tile_resource: Option<EResource>) -> Option<EResource> {
		None
	}

	fn pass_relatives(&self) -> &'static [(i32, i32)] {
		&[(0, 1), (0, -1), (1, 0), (-1, 0)]
	}
}

#[derive(Clone, Debug)]
pub enum EBuilding {
	Nothing(Nothing),
	SmallExtractor(SmallExtractor),
	DebugConsumer(DebugConsumer),
	Conveyor(Conveyor),
}
impl EBuilding {
	pub const fn nothing() -> Self {
		Self::Nothing(Nothing)
	}
	pub fn small_extractor() -> Self {
		Self::SmallExtractor(SmallExtractor::new())
	}
	pub const fn debug_consumer() -> Self {
		Self::DebugConsumer(DebugConsumer)
	}
	pub fn conveyor(dir: Direction) -> Self {
		Self::Conveyor(Conveyor::new(dir))
	}
}
impl Default for EBuilding {
	fn default() -> Self {
		Self::Nothing(Default::default())
	}
}
impl Building for EBuilding {
	fn name(&self) -> Cow<'static, str> {
		match self {
			Self::Nothing(a) => a.name(),
			Self::SmallExtractor(a) => a.name(),
			Self::DebugConsumer(a) => a.name(),
			Self::Conveyor(a) => a.name(),
		}
	}
	fn texture_id(&self) -> TextureID {
		match self {
			Self::Nothing(a) => a.texture_id(),
			Self::SmallExtractor(a) => a.texture_id(),
			Self::DebugConsumer(a) => a.texture_id(),
			Self::Conveyor(a) => a.texture_id(),
		}
	}

	fn render<'a>(&'a self, textures: &'a Textures) -> impl Layable + Clone + Debug + 'a {
		match self {
			Self::Nothing(a) => sui::custom(a.render(textures)),
			Self::SmallExtractor(a) => sui::custom(a.render(textures)),
			Self::DebugConsumer(a) => sui::custom(a.render(textures)),
			Self::Conveyor(a) => sui::custom(a.render(textures)),
		}
	}

	fn can_receive(&self, resource: &EResource) -> bool {
		match self {
			Self::Nothing(a) => a.can_receive(resource),
			Self::SmallExtractor(a) => a.can_receive(resource),
			Self::DebugConsumer(a) => a.can_receive(resource),
			Self::Conveyor(a) => a.can_receive(resource),
		}
	}
	fn receive(&mut self, resource: EResource) {
		match self {
			Self::Nothing(a) => a.receive(resource),
			Self::SmallExtractor(a) => a.receive(resource),
			Self::DebugConsumer(a) => a.receive(resource),
			Self::Conveyor(a) => a.receive(resource),
		}
	}

	fn needs_poll(&self) -> bool {
		match self {
			Self::Nothing(a) => a.needs_poll(),
			Self::SmallExtractor(a) => a.needs_poll(),
			Self::DebugConsumer(a) => a.needs_poll(),
			Self::Conveyor(a) => a.needs_poll(),
		}
	}
	fn resource_sample(&self, tile_resource: Option<EResource>) -> Option<EResource> {
		match self {
			Self::Nothing(a) => a.resource_sample(tile_resource),
			Self::SmallExtractor(a) => a.resource_sample(tile_resource),
			Self::DebugConsumer(a) => a.resource_sample(tile_resource),
			Self::Conveyor(a) => a.resource_sample(tile_resource),
		}
	}
	fn poll_resource(&mut self, tile_resource: Option<EResource>) -> Option<EResource> {
		match self {
			Self::Nothing(a) => a.poll_resource(tile_resource),
			Self::SmallExtractor(a) => a.poll_resource(tile_resource),
			Self::DebugConsumer(a) => a.poll_resource(tile_resource),
			Self::Conveyor(a) => a.poll_resource(tile_resource),
		}
	}

	fn pass_relatives(&self) -> &'static [(i32, i32)] {
		match self {
			Self::Nothing(a) => a.pass_relatives(),
			Self::SmallExtractor(a) => a.pass_relatives(),
			Self::DebugConsumer(a) => a.pass_relatives(),
			Self::Conveyor(a) => a.pass_relatives(),
		}
	}
}

#[derive(Clone, Debug, Default)]
pub struct Nothing;
impl Building for Nothing {
	fn name(&self) -> Cow<'static, str> {
		"nothing".into()
	}
	fn render<'a>(&'a self, _textures: &'a Textures) -> impl Layable + Clone + Debug + 'a {
		sui::comp::Space::new(0, 0)
	}

	fn texture_id(&self) -> TextureID {
		TextureID::Transparent
	}
}

#[derive(Clone, Debug, Default)]
pub struct BuildingsMap {
	buildings: [[EBuilding; SIZE]; SIZE],
	/// syntax: (source_position we poll the resource from, target building that'll receive the polled event) \
	/// this is only used internally while ticking and between ticks it's empty
	moves_queue: VecDeque<((i32, i32), (i32, i32))>,
}
impl BuildingsMap {
	pub fn tick(
		&mut self,
		mut tile_resource_at: impl FnMut((usize, usize)) -> Option<EResource>,
	) -> () {
		// warning: self.moves_queue gets taken as moves_queue and put back into self.moves_queue at the end of this function
		let mut moves_queue = std::mem::take(&mut self.moves_queue);

		for (pos, building) in self.iter() {
			if !building.needs_poll() {
				continue;
			}

			let dirs = building.pass_relatives();
			let pass_candidates = dirs
				.iter()
				.copied()
				.map(|(rx, ry)| (pos.0 + rx, pos.1 + ry));
			let pass_candidates = pass_candidates.filter_map(|pos| self.at(pos).map(|b| (pos, b)));

			let tile_resource = tile_resource_at((pos.0 as _, pos.1 as _));
			let resource_sample = building.resource_sample(tile_resource);

			if let Some(resource_sample) = &resource_sample {
				let pass_candidates = pass_candidates
					.filter(|(_, b)| b.can_receive(resource_sample))
					.map(|a| a.0);

				for pass_target in pass_candidates {
					if moves_queue
						.iter()
						.filter(|(_src, dst)| pass_target == *dst)
						.count() == 0
					{
						moves_queue.push_back((pos, pass_target));
						break;
					}
				}
			}
		}

		for (source_pos, target_pos) in moves_queue.drain(..) {
			let resource = {
				let source = match self.at_mut(source_pos) {
					Some(a) => a,
					None => continue,
				};

				let tile_resource = tile_resource_at((source_pos.0 as _, source_pos.1 as _));
				let resource = match source.poll_resource(tile_resource) {
					Some(a) => a,
					None => continue,
				};
				resource
			};

			let target = match self.at_mut(target_pos) {
				Some(a) => a,
				None => continue,
			};
			target.receive(resource);
		}

		self.moves_queue = moves_queue;
	}

	pub fn at(&self, pos: (i32, i32)) -> Option<&EBuilding> {
		if pos.0 < 0 || pos.1 < 0 || pos.0 >= SIZE as _ || pos.1 >= SIZE as _ {
			return None;
		}
		Some(&self.buildings[pos.0 as usize][pos.1 as usize])
	}
	pub fn at_mut(&mut self, pos: (i32, i32)) -> Option<&mut EBuilding> {
		if pos.0 < 0 || pos.1 < 0 || pos.0 >= SIZE as _ || pos.1 >= SIZE as _ {
			return None;
		}
		Some(&mut self.buildings[pos.0 as usize][pos.1 as usize])
	}

	pub fn iter<'a>(&'a self) -> impl Iterator<Item = ((i32, i32), &'a EBuilding)> + 'a {
		self.buildings
			.iter()
			.enumerate()
			.map(|(x, a)| {
				a.iter()
					.enumerate()
					.map(move |(y, a)| ((x as _, y as _), a))
			})
			.flatten()
	}
	pub fn iter_mut<'a>(
		&'a mut self,
	) -> impl Iterator<Item = ((i32, i32), &'a mut EBuilding)> + 'a {
		self.buildings
			.iter_mut()
			.enumerate()
			.map(|(x, a)| {
				a.iter_mut()
					.enumerate()
					.map(move |(y, a)| ((x as _, y as _), a))
			})
			.flatten()
	}

	pub fn render<'a, 'b: 'a>(&'a self, textures: &'b Textures) -> BuildingsRenderer<'a, 'b> {
		BuildingsRenderer::new(self, textures)
	}
}

#[derive(Clone, Debug)]
pub struct BuildingsRenderer<'a, 'b> {
	textures: &'b Textures,
	buildings: &'a BuildingsMap,
}
impl<'a, 'b> BuildingsRenderer<'a, 'b> {
	pub fn new(buildings: &'a BuildingsMap, textures: &'b Textures) -> Self {
		Self {
			textures,
			buildings,
		}
	}
}
impl<'a, 'b> Layable for BuildingsRenderer<'a, 'b> {
	fn size(&self) -> (i32, i32) {
		let size = SIZE as i32 * TILE_RENDER_SIZE as i32;
		(size, size)
	}
	fn render(&self, d: &mut sui::Handle, det: sui::Details, scale: f32) {
		render::draw_buildings(d, &self.buildings, &self.textures, det.x, det.y, scale)
	}
}
