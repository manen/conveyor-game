// pub mod small_extractor;
// pub use small_extractor::*;

use std::{borrow::Cow, fmt::Debug};

use sui::{Layable, LayableExt};

use crate::{
	textures::{TextureID, Textures},
	utils::Direction,
	world::{EResource, render::TILE_RENDER_SIZE},
};

mod conveyor;
pub use conveyor::*;
mod junction;
pub use junction::*;
mod router;
pub use router::*;
mod small_extractor;
pub use small_extractor::*;
mod debug_consumer;
pub use debug_consumer::*;
mod channel_consumer;
pub use channel_consumer::*;

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

	/// used to render an image of the building statically \
	/// the returned layable can't depend on any lifetime
	fn tool_icon_render(&self, textures: &Textures) -> impl Layable + Clone + Debug + 'static {
		textures
			.texture_for(self.texture_id())
			.expect("texture for building isn't loaded")
			.clone()
			.fix_wh_square(64)
	}

	fn can_receive(&self, from: Option<Direction>) -> bool {
		false
	}
	// returns how many of the given resource it can receive right now
	fn capacity_for(&self, resource: &EResource, from: Option<Direction>) -> i32 {
		0
	}
	fn receive(&mut self, resource: EResource, from: Option<Direction>) {}

	/// [Self::poll_resource], without advancing any internal timers or anything
	fn resource_sample(
		&self,
		tile_resource: Option<EResource>,
		to: Option<Direction>,
	) -> Option<EResource> {
		None
	}
	// polling is how you generate new shit
	fn needs_poll(&self) -> bool {
		false
	}
	fn poll_resource(
		&mut self,
		tile_resource: Option<EResource>,
		to: Option<Direction>,
	) -> Option<EResource> {
		None
	}

	/// higher is better/more favorable
	fn rank_pass_source(&self, relative_pos: (i32, i32)) -> i32 {
		1
	}

	/// even though this can return any number as a relative, if it's not a direction it will not go through by
	/// the current implementation
	fn pass_relatives(&self) -> &'static [(i32, i32)] {
		&[(0, 1), (0, -1), (1, 0), (-1, 0)]
	}
	/// lets the building pick which target candidate it'd like to pass resources to
	fn confirm_pass_relatives(
		&mut self,
		available_directions: &[(i32, i32)],
	) -> Option<(i32, i32)> {
		available_directions.iter().cloned().last()
	}

	/// if true, this building can't be removed by the standard eraser tool
	fn is_protected(&self) -> bool {
		false
	}
}

#[derive(Clone, Debug)]
pub enum EBuilding {
	Nothing(Nothing),
	SmallExtractor(SmallExtractor),
	DebugConsumer(DebugConsumer),
	ChannelConsumer(ChannelConsumer),
	Conveyor(Conveyor),
	Junction(Junction),
	Router(Router),
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
	pub fn junction() -> Self {
		Self::Junction(Junction::default())
	}
	pub fn router() -> Self {
		Self::Router(Router::default())
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
			Self::ChannelConsumer(a) => a.name(),
			Self::Conveyor(a) => a.name(),
			Self::Junction(a) => a.name(),
			Self::Router(a) => a.name(),
		}
	}
	fn texture_id(&self) -> TextureID {
		match self {
			Self::Nothing(a) => a.texture_id(),
			Self::SmallExtractor(a) => a.texture_id(),
			Self::DebugConsumer(a) => a.texture_id(),
			Self::ChannelConsumer(a) => a.texture_id(),
			Self::Conveyor(a) => a.texture_id(),
			Self::Junction(a) => a.texture_id(),
			Self::Router(a) => a.texture_id(),
		}
	}

	fn render<'a>(&'a self, textures: &'a Textures) -> impl Layable + Clone + Debug + 'a {
		match self {
			Self::Nothing(a) => sui::custom(a.render(textures)),
			Self::SmallExtractor(a) => sui::custom(a.render(textures)),
			Self::DebugConsumer(a) => sui::custom(a.render(textures)),
			Self::ChannelConsumer(a) => sui::custom(a.render(textures)),
			Self::Conveyor(a) => sui::custom(a.render(textures)),
			Self::Junction(a) => sui::custom(a.render(textures)),
			Self::Router(a) => sui::custom(a.render(textures)),
		}
	}
	fn tool_icon_render(&self, textures: &Textures) -> impl Layable + Clone + Debug + 'static {
		match self {
			Self::Nothing(a) => sui::custom(a.tool_icon_render(textures)),
			Self::SmallExtractor(a) => sui::custom(a.tool_icon_render(textures)),
			Self::DebugConsumer(a) => sui::custom(a.tool_icon_render(textures)),
			Self::ChannelConsumer(a) => sui::custom(a.tool_icon_render(textures)),
			Self::Conveyor(a) => sui::custom(a.tool_icon_render(textures)),
			Self::Junction(a) => sui::custom(a.tool_icon_render(textures)),
			Self::Router(a) => sui::custom(a.tool_icon_render(textures)),
		}
	}

	fn can_receive(&self, from: Option<Direction>) -> bool {
		match self {
			Self::Nothing(a) => a.can_receive(from),
			Self::SmallExtractor(a) => a.can_receive(from),
			Self::DebugConsumer(a) => a.can_receive(from),
			Self::ChannelConsumer(a) => a.can_receive(from),
			Self::Conveyor(a) => a.can_receive(from),
			Self::Junction(a) => a.can_receive(from),
			Self::Router(a) => a.can_receive(from),
		}
	}
	fn capacity_for(&self, resource: &EResource, from: Option<Direction>) -> i32 {
		match self {
			Self::Nothing(a) => a.capacity_for(resource, from),
			Self::SmallExtractor(a) => a.capacity_for(resource, from),
			Self::DebugConsumer(a) => a.capacity_for(resource, from),
			Self::ChannelConsumer(a) => a.capacity_for(resource, from),
			Self::Conveyor(a) => a.capacity_for(resource, from),
			Self::Junction(a) => a.capacity_for(resource, from),
			Self::Router(a) => a.capacity_for(resource, from),
		}
	}
	fn receive(&mut self, resource: EResource, from: Option<Direction>) {
		match self {
			Self::Nothing(a) => a.receive(resource, from),
			Self::SmallExtractor(a) => a.receive(resource, from),
			Self::DebugConsumer(a) => a.receive(resource, from),
			Self::ChannelConsumer(a) => a.receive(resource, from),
			Self::Conveyor(a) => a.receive(resource, from),
			Self::Junction(a) => a.receive(resource, from),
			Self::Router(a) => a.receive(resource, from),
		}
	}

	fn needs_poll(&self) -> bool {
		match self {
			Self::Nothing(a) => a.needs_poll(),
			Self::SmallExtractor(a) => a.needs_poll(),
			Self::DebugConsumer(a) => a.needs_poll(),
			Self::ChannelConsumer(a) => a.needs_poll(),
			Self::Conveyor(a) => a.needs_poll(),
			Self::Junction(a) => a.needs_poll(),
			Self::Router(a) => a.needs_poll(),
		}
	}
	fn resource_sample(
		&self,
		tile_resource: Option<EResource>,
		to: Option<Direction>,
	) -> Option<EResource> {
		match self {
			Self::Nothing(a) => a.resource_sample(tile_resource, to),
			Self::SmallExtractor(a) => a.resource_sample(tile_resource, to),
			Self::DebugConsumer(a) => a.resource_sample(tile_resource, to),
			Self::ChannelConsumer(a) => a.resource_sample(tile_resource, to),
			Self::Conveyor(a) => a.resource_sample(tile_resource, to),
			Self::Junction(a) => a.resource_sample(tile_resource, to),
			Self::Router(a) => a.resource_sample(tile_resource, to),
		}
	}
	fn poll_resource(
		&mut self,
		tile_resource: Option<EResource>,
		to: Option<Direction>,
	) -> Option<EResource> {
		match self {
			Self::Nothing(a) => a.poll_resource(tile_resource, to),
			Self::SmallExtractor(a) => a.poll_resource(tile_resource, to),
			Self::DebugConsumer(a) => a.poll_resource(tile_resource, to),
			Self::ChannelConsumer(a) => a.poll_resource(tile_resource, to),
			Self::Conveyor(a) => a.poll_resource(tile_resource, to),
			Self::Junction(a) => a.poll_resource(tile_resource, to),
			Self::Router(a) => a.poll_resource(tile_resource, to),
		}
	}

	fn pass_relatives(&self) -> &'static [(i32, i32)] {
		match self {
			Self::Nothing(a) => a.pass_relatives(),
			Self::SmallExtractor(a) => a.pass_relatives(),
			Self::DebugConsumer(a) => a.pass_relatives(),
			Self::ChannelConsumer(a) => a.pass_relatives(),
			Self::Conveyor(a) => a.pass_relatives(),
			Self::Junction(a) => a.pass_relatives(),
			Self::Router(a) => a.pass_relatives(),
		}
	}
	fn confirm_pass_relatives(&mut self, dirs: &[(i32, i32)]) -> Option<(i32, i32)> {
		match self {
			Self::Nothing(a) => a.confirm_pass_relatives(dirs),
			Self::SmallExtractor(a) => a.confirm_pass_relatives(dirs),
			Self::DebugConsumer(a) => a.confirm_pass_relatives(dirs),
			Self::ChannelConsumer(a) => a.confirm_pass_relatives(dirs),
			Self::Conveyor(a) => a.confirm_pass_relatives(dirs),
			Self::Junction(a) => a.confirm_pass_relatives(dirs),
			Self::Router(a) => a.confirm_pass_relatives(dirs),
		}
	}
	fn rank_pass_source(&self, relative_pos: (i32, i32)) -> i32 {
		match self {
			Self::Nothing(a) => a.rank_pass_source(relative_pos),
			Self::SmallExtractor(a) => a.rank_pass_source(relative_pos),
			Self::DebugConsumer(a) => a.rank_pass_source(relative_pos),
			Self::ChannelConsumer(a) => a.rank_pass_source(relative_pos),
			Self::Conveyor(a) => a.rank_pass_source(relative_pos),
			Self::Junction(a) => a.rank_pass_source(relative_pos),
			Self::Router(a) => a.rank_pass_source(relative_pos),
		}
	}

	fn is_protected(&self) -> bool {
		match self {
			Self::Nothing(a) => a.is_protected(),
			Self::SmallExtractor(a) => a.is_protected(),
			Self::DebugConsumer(a) => a.is_protected(),
			Self::ChannelConsumer(a) => a.is_protected(),
			Self::Conveyor(a) => a.is_protected(),
			Self::Junction(a) => a.is_protected(),
			Self::Router(a) => a.is_protected(),
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
	fn render<'a>(&'a self, _textures: &'a Textures) -> impl Layable + Clone + Debug + 'a {
		sui::comp::Space::new(0, 0)
	}

	fn tool_icon_render(&self, textures: &Textures) -> impl Layable + Clone + Debug + 'static {
		textures.texture_for(TextureID::Eraser).cloned()
	}
}
