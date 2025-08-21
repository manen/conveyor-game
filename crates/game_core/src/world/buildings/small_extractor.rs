use std::time::{Duration, Instant};

use crate::{EResource, buildings::Building};
use textures::TextureID;
use utils::Direction;

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct SmallExtractor {
	#[serde(skip, default = "Instant::now")]
	last_extract: Instant,

	protected: bool,
}
impl SmallExtractor {
	pub fn new() -> Self {
		Self {
			last_extract: Instant::now(),
			protected: false,
		}
	}
}
impl Building for SmallExtractor {
	fn name(&self) -> std::borrow::Cow<'static, str> {
		"small extractor".into()
	}
	fn texture_id(&self) -> TextureID {
		TextureID::SmallExtractor
	}

	fn needs_poll(&self) -> bool {
		self.last_extract.elapsed() > Duration::from_millis(750)
	}
	fn resource_sample(
		&self,
		tile_resource: Option<EResource>,
		_to: Option<Direction>,
	) -> Option<EResource> {
		tile_resource
	}
	fn poll_resource(
		&mut self,
		tile_resource: Option<EResource>,
		_to: Option<Direction>,
	) -> Option<EResource> {
		if self.needs_poll() {
			self.last_extract = Instant::now();
			self.resource_sample(tile_resource, _to)
		} else {
			None
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
