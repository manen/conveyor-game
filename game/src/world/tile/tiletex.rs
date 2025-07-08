use std::borrow::Cow;

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum TileTexture {
	None,
	Stone,
}
impl TileTexture {
	/// none just becomes transparent
	pub const fn resource_path(&self) -> Option<Cow<'static, str>> {
		match self {
			TileTexture::None => None,
			TileTexture::Stone => Some(Cow::Borrowed(
				"https://pbs.twimg.com/media/DOvSaYKX4AERSq2.jpg",
			)),
		}
	}
}

pub const fn all_textures() -> &'static [TileTexture] {
	&[TileTexture::None, TileTexture::Stone]
}
