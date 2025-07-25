use std::borrow::Cow;

use crate::textures::TextureID;

mod iron;
pub use iron::*;
mod coal;
pub use coal::*;

pub trait Resource {
	fn name(&self) -> Cow<'static, str>;
	fn texture_id(&self) -> TextureID;
}

#[derive(Clone, Debug)]
pub enum EResource {
	Coal(Coal),
	RawIron(RawIron),
}
impl EResource {
	pub fn coal() -> Self {
		Self::Coal(Coal)
	}
	pub fn raw_iron() -> Self {
		Self::RawIron(RawIron)
	}
}
