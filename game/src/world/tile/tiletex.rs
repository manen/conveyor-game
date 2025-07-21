use std::borrow::Cow;

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum TileTexture {
	Stone,
}
impl TileTexture {
	/// none just becomes transparent
	pub const fn resource_path(&self) -> Cow<'static, str> {
		match self {
			TileTexture::Stone => Cow::Borrowed("https://pbs.twimg.com/media/DOvSaYKX4AERSq2.jpg"),
		}
	}
}

pub const fn all_textures() -> &'static [TileTexture] {
	&[TileTexture::Stone]
}
