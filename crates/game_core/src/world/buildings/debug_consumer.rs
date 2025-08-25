use crate::buildings::Building;
use textures::TextureID;
use utils::Direction;

#[derive(Copy, Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct DebugConsumer;
impl Building for DebugConsumer {
	fn name(&self) -> std::borrow::Cow<'static, str> {
		"debug consumer".into()
	}
	fn texture_id(&self) -> TextureID {
		TextureID::RawIron
	}

	fn can_receive(&self, _from: Option<Direction>) -> bool {
		true
	}
	fn capacity_for(&self, _resource: &crate::EResource, _from: Option<Direction>) -> i32 {
		10
	}
	fn receive(&mut self, resource: crate::EResource, _from: Option<Direction>) {
		mklogger::println!("debug consumer dropped {resource:?}")
	}
}
