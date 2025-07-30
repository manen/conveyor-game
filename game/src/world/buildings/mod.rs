// pub mod small_extractor;
// pub use small_extractor::*;

use std::{borrow::Cow, collections::HashMap, fmt::Debug};

use sui::Layable;

use crate::{
	textures::{TextureID, Textures},
	utils::{Direction, MultiMap},
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

#[allow(unused)]
pub trait Building {
	fn name(&self) -> Cow<'static, str>;
	fn texture_id(&self) -> TextureID;

	/// the scale in the returned Layable should be ignored as it's always 1.0 \
	/// det.aw and det.ah is the tile width&height, go off that
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
				self.textures.render(d, det, &self.texture_id);
			}
		}

		DefaultBuildingRender {
			texture_id: self.texture_id(),
			textures,
		}
	}

	// returns how many of the given resource it can receive right now
	fn can_receive(&self) -> bool {
		false
	}
	fn capacity_for(&self, resource: &EResource) -> i32 {
		0
	}
	fn receive(&mut self, resource: EResource) {}

	/// [Self::poll_resource], without advancing any internal timers or anything
	fn resource_sample(&self, tile_resource: Option<EResource>) -> Option<EResource> {
		None
	}
	// polling is how you generate new shit
	fn needs_poll(&self) -> bool {
		false
	}
	fn poll_resource(&mut self, tile_resource: Option<EResource>) -> Option<EResource> {
		None
	}

	/// higher is better/more favorable
	fn rank_pass_source(&self, relative_pos: (i32, i32)) -> i32 {
		1
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

	fn can_receive(&self) -> bool {
		match self {
			Self::Nothing(a) => a.can_receive(),
			Self::SmallExtractor(a) => a.can_receive(),
			Self::DebugConsumer(a) => a.can_receive(),
			Self::Conveyor(a) => a.can_receive(),
		}
	}
	fn capacity_for(&self, resource: &EResource) -> i32 {
		match self {
			Self::Nothing(a) => a.capacity_for(resource),
			Self::SmallExtractor(a) => a.capacity_for(resource),
			Self::DebugConsumer(a) => a.capacity_for(resource),
			Self::Conveyor(a) => a.capacity_for(resource),
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
	fn rank_pass_source(&self, relative_pos: (i32, i32)) -> i32 {
		match self {
			Self::Nothing(a) => a.rank_pass_source(relative_pos),
			Self::SmallExtractor(a) => a.rank_pass_source(relative_pos),
			Self::DebugConsumer(a) => a.rank_pass_source(relative_pos),
			Self::Conveyor(a) => a.rank_pass_source(relative_pos),
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
	/// HashMap<target_position, Vec<source positions>>
	moves_queue: HashMap<(i32, i32), Vec<(i32, i32)>>,
}
impl BuildingsMap {
	pub fn tick(
		&mut self,
		mut tile_resource_at: impl FnMut((i32, i32)) -> Option<EResource>,
	) -> () {
		let mut tile_resource_at = |(x, y)| tile_resource_at((x as _, y as _));

		// warning: self.moves_queue gets taken as moves_queue and put back into self.moves_queue at the end of this function
		let mut moves_queue = std::mem::take(&mut self.moves_queue);

		// imma leave it at that but in this order some generators just randomly don't work?
		// only generators it seems like
		//
		// gl
		//
		// okay so it looks like it's not even the order it's broken with the sorting turned off and with the rewritten poller too
		// szoval kizarasos alapon csak maga a mover lehet de itt semmilyen indok nincs h ne mukodjon
		// logolas mint az allat mar mutatja a building debug info a koordikat szoval printelni ha extractort pollolunk meg ilyenek byeee

		// poll the resources from the source and push them into the target block
		let moves_total = moves_queue.multimap_drain_total();
		for (target_pos, source_pos) in moves_total {
			let mut f = || {
				let target = self.at(*target_pos)?;
				let source = self.at(source_pos)?;

				let tile_resource = tile_resource_at(source_pos);
				let sample = source.resource_sample(tile_resource.clone())?;
				let capacity = target.capacity_for(&sample);

				if capacity > 0 {
					let source = self.at_mut(source_pos)?;
					let resource = source.poll_resource(tile_resource)?;

					let target = self.at_mut(*target_pos)?;
					target.receive(resource);
				}

				Some(0)
			};
			match f() {
				Some(_) => {}
				None => {}
			}
		}

		// check for buildings that need polling and list
		for (source_pos, building) in self.iter() {
			if !building.needs_poll() {
				continue;
			}

			let relatives = building.pass_relatives();
			let target_poss = relatives
				.iter()
				.cloned()
				.map(|(rx, ry)| (source_pos.0 + rx, source_pos.1 + ry));

			for target_pos in target_poss {
				if self
					.at(target_pos)
					.map(|target| target.can_receive())
					.unwrap_or(false)
				{
					moves_queue.multimap_insert(target_pos, source_pos);
				}
			}
		}

		// sort incoming resources by the target building's preferences
		for (target_pos, source_poss) in moves_queue.iter_mut().filter(|(_, v)| !v.is_empty()) {
			let mut f = || {
				let target = self.at(*target_pos)?;

				source_poss.sort_by_key(|source_pos| {
					let rel_pos = (target_pos.0 - source_pos.0, target_pos.1 - source_pos.1);
					-target.rank_pass_source(rel_pos)
				});
				Some(1)
			};
			match f() {
				Some(_) => (),
				None => source_poss.clear(),
			}
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
