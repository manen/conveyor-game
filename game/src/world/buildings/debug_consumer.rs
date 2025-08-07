use crate::{textures::TextureID, utils::Direction, world::buildings::Building};

#[derive(Copy, Clone, Debug)]
pub struct DebugConsumer;
impl Building for DebugConsumer {
	fn name(&self) -> std::borrow::Cow<'static, str> {
		"debug consumer".into()
	}
	fn texture_id(&self) -> crate::textures::TextureID {
		TextureID::RawIron
	}

	fn can_receive(&self, _from: Option<Direction>) -> bool {
		true
	}
	fn capacity_for(&self, _resource: &crate::world::EResource, _from: Option<Direction>) -> i32 {
		10
	}
	fn receive(&mut self, resource: crate::world::EResource, _from: Option<Direction>) {
		println!("debug consumer dropped {resource:?}")
	}
}
