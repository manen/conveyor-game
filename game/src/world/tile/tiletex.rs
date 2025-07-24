use std::borrow::Cow;

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum TileTexture {
	Stone,
	IronOre,
	CoalOre,
}
impl TileTexture {
	/// none just becomes transparent
	pub const fn resource_path(&self) -> Cow<'static, str> {
		match self {
			TileTexture::Stone => Cow::Borrowed("https://pbs.twimg.com/media/DOvSaYKX4AERSq2.jpg"),
			TileTexture::IronOre => Cow::Borrowed(
				"https://cdn.modrinth.com/data/GaB6rnEA/images/b9f180ff26fc858341cf326197f3798cd8fb6bac.png",
			),
			TileTexture::CoalOre => Cow::Borrowed(
				"https://d31sxl6qgne2yj.cloudfront.net/wordpress/wp-content/uploads/20190102094854/Minecraft-Coal-Ore.jpg",
			),
		}
	}
}

pub const fn all_textures() -> &'static [TileTexture] {
	&[
		TileTexture::Stone,
		TileTexture::IronOre,
		TileTexture::CoalOre,
	]
}
