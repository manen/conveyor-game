// pub mod small_extractor;
// pub use small_extractor::*;

use std::{borrow::Cow, fmt::Debug};

use sui::Layable;

use crate::{
	textures::{TextureID, Textures},
	utils::Direction,
	world::{EResource, render::TILE_RENDER_SIZE},
};

mod small_extractor;
pub use small_extractor::*;
mod debug_consumer;
pub use debug_consumer::*;
mod conveyor;
pub use conveyor::*;

pub use super::maps::BuildingsMap;

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
