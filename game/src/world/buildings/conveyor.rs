use crate::{
	textures::TextureID,
	utils::Direction,
	world::{EResource, buildings::Building},
};

#[derive(Clone, Debug)]
pub struct Conveyor {
	pub dir: Direction,
	holding: Option<EResource>,
}
impl Conveyor {
	pub fn new(dir: Direction) -> Self {
		Self { dir, holding: None }
	}
}

impl Building for Conveyor {
	fn name(&self) -> std::borrow::Cow<'static, str> {
		"conveyor".into()
	}
	fn texture_id(&self) -> TextureID {
		TextureID::Coal
	}

	fn can_receive(&self, _resource: &EResource) -> bool {
		self.holding.is_none()
	}
	fn receive(&mut self, resource: EResource) {
		if self.can_receive(&resource) {
			self.holding = Some(resource)
		}
	}

	fn needs_poll(&self) -> bool {
		self.holding.is_some()
	}
	fn resource_sample(&self, _tile_resource: Option<EResource>) -> Option<EResource> {
		self.holding.clone()
	}
	fn poll_resource(&mut self, _tile_resource: Option<EResource>) -> Option<EResource> {
		self.holding.take()
	}

	fn pass_relatives(&self) -> &'static [(i32, i32)] {
		self.dir.rel_array()
	}
}

// TODO:
// - conveyor timing
// - only call poll_resource if we're sure we can pass the item somewhere (to avoid items straightup disappearing if they can't go anywhere)
