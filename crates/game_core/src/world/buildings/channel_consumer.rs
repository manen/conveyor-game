use tokio::sync::mpsc;

use crate::{EResource, buildings::Building};
use textures::TextureID;
use utils::Direction;

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct ChannelConsumer {
	#[serde(skip, default)]
	tx: Option<mpsc::Sender<EResource>>,
	pub protected: bool,
}
impl ChannelConsumer {
	pub fn new() -> (Self, mpsc::Receiver<EResource>) {
		let (tx, rx) = mpsc::channel(20);
		let consumer = Self {
			tx: Some(tx),
			protected: false,
		};

		(consumer, rx)
	}

	fn capacity_unwrapped(&self) -> usize {
		self.tx.as_ref().map(|tx| tx.capacity()).unwrap_or_default()
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
		self.capacity_unwrapped() > 0
	}
	fn capacity_for(&self, _resource: &EResource, _from: Option<Direction>) -> i32 {
		self.capacity_unwrapped() as i32
	}
	fn receive(&mut self, resource: EResource, _from: Option<Direction>) {
		let mut drop_tx = false;
		if let Some(tx) = &mut self.tx {
			match tx.try_send(resource) {
				Ok(a) => a,
				Err(mpsc::error::TrySendError::Closed(_)) => drop_tx = true,
				Err(_) => {}
			}
		}
		if drop_tx {
			self.tx = None;
			self.protected = false;
		}
	}

	fn is_protected(&self) -> bool {
		self.protected
	}
	fn set_protected(&mut self, protected: bool) -> Result<(), ()> {
		self.protected = protected;
		Ok(())
	}
}

// pub mod serialize {
// 	use super::super::EBuilding;
// 	use serde::{Deserializer, Serialize, Serializer};

// 	pub fn serialize<T, S: Serializer>(_: &EBuilding, serializer: S) -> Result<S::Ok, S::Error> {
// 		println!("saving ChannelConsumer as Nothing");
// 		EBuilding::nothing().serialize(serializer)
// 	}
// 	pub fn deserialize<'de, D: Deserializer<'de>>(_: D) -> Result<EBuilding, D::Error> {
// 		println!("loading ChannelConsumer as Nothing");
// 		Ok(EBuilding::nothing())
// 	}
// }
