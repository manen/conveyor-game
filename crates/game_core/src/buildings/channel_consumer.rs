use tokio::sync::mpsc::{Receiver, Sender};

use crate::{EResource, buildings::Building};
use textures::TextureID;
use utils::Direction;

#[derive(Clone, Debug)]
pub struct ChannelConsumer {
	tx: Sender<EResource>,
	pub protected: bool,
}
impl ChannelConsumer {
	pub fn new() -> (Self, Receiver<EResource>) {
		let (tx, rx) = tokio::sync::mpsc::channel(20);
		let consumer = Self {
			tx,
			protected: false,
		};

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

	fn can_receive(&self, _from: Option<Direction>) -> bool {
		self.tx.capacity() > 0
	}
	fn capacity_for(&self, _resource: &EResource, _from: Option<Direction>) -> i32 {
		self.tx.capacity() as i32
	}
	fn receive(&mut self, resource: EResource, _from: Option<Direction>) {
		let _ = self.tx.try_send(resource);
	}

	fn is_protected(&self) -> bool {
		self.protected
	}
	fn set_protected(&mut self, protected: bool) -> Result<(), ()> {
		self.protected = protected;
		Ok(())
	}
}
