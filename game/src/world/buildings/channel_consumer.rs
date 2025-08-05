use tokio::sync::mpsc::{Receiver, Sender};

use crate::{
	textures::TextureID,
	world::{EResource, buildings::Building},
};

#[derive(Clone, Debug)]
pub struct ChannelConsumer {
	tx: Sender<EResource>,
}
impl ChannelConsumer {
	pub fn new() -> (Self, Receiver<EResource>) {
		let (tx, rx) = tokio::sync::mpsc::channel(20);
		let consumer = Self { tx };

		(consumer, rx)
	}
}

impl Building for ChannelConsumer {
	fn name(&self) -> std::borrow::Cow<'static, str> {
		"channel consumer".into()
	}
	fn texture_id(&self) -> TextureID {
		TextureID::ChannelConsumer
	}

	fn can_receive(&self) -> bool {
		self.tx.capacity() > 0
	}
	fn capacity_for(&self, _resource: &EResource) -> i32 {
		self.tx.capacity() as i32
	}
	fn receive(&mut self, resource: EResource) {
		let _ = self.tx.try_send(resource);
	}
}
