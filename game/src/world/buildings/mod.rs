// pub mod small_extractor;
// pub use small_extractor::*;

use std::{borrow::Cow, collections::VecDeque};

use sui::Layable;

use crate::{
	textures::{TextureID, Textures},
	world::{
		EResource,
		render::{self, TILE_RENDER_SIZE},
		tile,
		tilemap::SIZE,
	},
};

mod small_extractor;
pub use small_extractor::*;
mod debug_consumer;
pub use debug_consumer::*;

pub trait Building {
	fn name(&self) -> Cow<'static, str>;
	fn texture_id(&self) -> TextureID;

	// receiving is how shit gets passed
	fn can_receive(&self, _resource: &EResource) -> bool {
		false
	}
	fn receive(&mut self, _resource: EResource) {}

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
}
impl EBuilding {
	pub fn nothing() -> Self {
		Self::Nothing(Nothing)
	}
	pub fn small_extractor() -> Self {
		Self::SmallExtractor(SmallExtractor::new())
	}
	pub fn debug_consumer() -> Self {
		Self::DebugConsumer(DebugConsumer)
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
		}
	}
	fn texture_id(&self) -> TextureID {
		match self {
			Self::Nothing(a) => a.texture_id(),
			Self::SmallExtractor(a) => a.texture_id(),
			Self::DebugConsumer(a) => a.texture_id(),
		}
	}

	fn can_receive(&self, resource: &EResource) -> bool {
		match self {
			Self::Nothing(a) => a.can_receive(resource),
			Self::SmallExtractor(a) => a.can_receive(resource),
			Self::DebugConsumer(a) => a.can_receive(resource),
		}
	}
	fn receive(&mut self, resource: EResource) {
		match self {
			Self::Nothing(a) => a.receive(resource),
			Self::SmallExtractor(a) => a.receive(resource),
			Self::DebugConsumer(a) => a.receive(resource),
		}
	}

	fn needs_poll(&self) -> bool {
		match self {
			Self::Nothing(a) => a.needs_poll(),
			Self::SmallExtractor(a) => a.needs_poll(),
			Self::DebugConsumer(a) => a.needs_poll(),
		}
	}
	fn poll_resource(&mut self, tile_resource: Option<EResource>) -> Option<EResource> {
		match self {
			Self::Nothing(a) => a.poll_resource(tile_resource),
			Self::SmallExtractor(a) => a.poll_resource(tile_resource),
			Self::DebugConsumer(a) => a.poll_resource(tile_resource),
		}
	}

	fn pass_relatives(&self) -> &'static [(i32, i32)] {
		match self {
			Self::Nothing(a) => a.pass_relatives(),
			Self::SmallExtractor(a) => a.pass_relatives(),
			Self::DebugConsumer(a) => a.pass_relatives(),
		}
	}
}

#[derive(Clone, Debug, Default)]
pub struct Nothing;
impl Building for Nothing {
	fn name(&self) -> Cow<'static, str> {
		"nothing".into()
	}
	fn texture_id(&self) -> TextureID {
		TextureID::Transparent
	}
}

#[derive(Clone, Debug, Default)]
pub struct BuildingsMap {
	buildings: [[EBuilding; SIZE]; SIZE],
	/// syntax: (source_position, resource)
	moves_queue: VecDeque<((i32, i32), EResource)>,
}
impl BuildingsMap {
	pub fn tick(&mut self, mut tile_resource_at: impl FnMut((usize, usize)) -> Option<EResource>) {
		//! warning: self.moves_queue gets taken as moves_queue and put back into self.moves_queue at the end of this function
		let mut moves_queue = std::mem::take(&mut self.moves_queue);

		for (pos, resource) in moves_queue.drain(..) {
			let directions = match self.at_mut(pos) {
				Some(a) => a.pass_relatives(),
				None => continue,
			};
			for pos_rel in directions.iter().copied() {
				let pos = (pos.0 + pos_rel.0, pos.1 + pos_rel.1);
				let block = if let Some(at) = self.at(pos) {
					at
				} else {
					continue;
				};

				// the very first block in the building's requested pass directions that'll take the resource
				// will get the resource
				if block.can_receive(&resource) {
					self.at_mut(pos)
						.expect("could take building as immutable, can't as mutable")
						.receive(resource);
					break;
				}
			}
		}

		// let to_tick = self
		// 	.buildings
		// 	.iter_mut()
		// 	.enumerate()
		// 	.map(|(x, a)| a.iter_mut().enumerate().map(move |(y, a)| ((x, y), a)))
		// 	.flatten()
		// 	.filter(|(_, b)| b.needs_poll());
		// let to_tick = to_tick.filter_map(|((x, y), building)| {
		// 	building
		// 		.poll_resource(tile_resource_at((x, y)))
		// 		.map(|res| ((x as i32, y as i32), res))
		// });

		// one hell of an iterator huh

		for (pos, building) in self.iter_mut() {
			if !building.needs_poll() {
				continue;
			}
			let tile_resource = tile_resource_at((pos.0 as _, pos.1 as _));
			let polled_resource = building.poll_resource(tile_resource);

			if let Some(polled_resource) = polled_resource {
				moves_queue.push_back((pos, polled_resource));
			}
		}
		self.moves_queue = moves_queue;
	}

	pub fn at(&self, pos: (i32, i32)) -> Option<&EBuilding> {
		if pos.0 < 0 || pos.1 < 0 || pos.0 > SIZE as _ || pos.1 > SIZE as _ {
			return None;
		}
		Some(&self.buildings[pos.0 as usize][pos.1 as usize])
	}
	pub fn at_mut(&mut self, pos: (i32, i32)) -> Option<&mut EBuilding> {
		if pos.0 < 0 || pos.1 < 0 || pos.0 > SIZE as _ || pos.1 > SIZE as _ {
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
