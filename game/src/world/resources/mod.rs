use std::borrow::Cow;

use crate::textures::TextureID;

pub trait Resource {
	fn name(&self) -> Cow<'static, str>;
	fn texture_id(&self) -> TextureID;
}

#[derive(Clone, Debug)]
pub enum EResource {}
