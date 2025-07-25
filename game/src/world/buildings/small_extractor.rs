use std::time::{Duration, Instant};

use crate::{
	textures::TextureID,
	world::{EResource, buildings::Building},
};

#[derive(Clone, Debug)]
pub struct SmallExtractor {
	last_extract: Instant,
}
impl SmallExtractor {
	pub fn new() -> Self {
		Self {
			last_extract: Instant::now(),
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
	fn poll_resource(&mut self, tile_resource: Option<EResource>) -> Option<EResource> {
		if self.needs_poll() {
			self.last_extract = Instant::now();
			tile_resource
		} else {
			None
		}
	}
}
