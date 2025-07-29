use crate::{textures::TextureID, world::buildings::Building};

#[derive(Copy, Clone, Debug)]
pub struct DebugConsumer;
impl Building for DebugConsumer {
	fn name(&self) -> std::borrow::Cow<'static, str> {
		"debug consumer".into()
	}
	fn texture_id(&self) -> crate::textures::TextureID {
		TextureID::RawIron
	}

	fn capacity_for(&self, _resource: &crate::world::EResource) -> i32 {
		10
	}
	fn receive(&mut self, resource: crate::world::EResource) {
		println!("debug consumer dropped {resource:?}")
	}
}
