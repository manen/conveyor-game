use std::{borrow::Cow, collections::HashMap};

use anyhow::Context;
use asset_provider::Assets;
use asset_provider_image::{AssetsExt, ImageExt, image::DynamicImage};
use futures::{Stream, stream::FuturesUnordered};
use strum::{EnumIter, IntoEnumIterator};
use sui::tex::Texture;

/// an enum for every texture we can use \
/// extensible in the future by adding an Other(u64) and have some sort of setup that hashes their
/// resource_path, just so it's faster \
/// this id is used a lot in every tick
#[derive(Clone, Debug, PartialEq, Eq, Hash, EnumIter)]
pub enum TextureID {
	Transparent,

	SmallExtractor,

	Stone,
	IronOre,
	CoalOre,

	Coal,
	RawIron,
}
impl TextureID {
	/// none just becomes transparent
	pub const fn resource_path(&self) -> Cow<'static, str> {
		match self {
			TextureID::Transparent => Cow::Borrowed("textures/transparent.png"),

			TextureID::SmallExtractor => Cow::Borrowed(
				"https://static.wikia.nocookie.net/minecraft_gamepedia/images/e/e7/Diamond_Pickaxe_JE3_BE3.png/revision/latest/scale-to-width/360?cb=20250628224016",
			),

			TextureID::Stone => Cow::Borrowed("https://pbs.twimg.com/media/DOvSaYKX4AERSq2.jpg"),
			TextureID::IronOre => Cow::Borrowed(
				"https://cdn.modrinth.com/data/GaB6rnEA/images/b9f180ff26fc858341cf326197f3798cd8fb6bac.png",
			),
			TextureID::CoalOre => Cow::Borrowed(
				"https://d31sxl6qgne2yj.cloudfront.net/wordpress/wp-content/uploads/20190102094854/Minecraft-Coal-Ore.jpg",
			),
			TextureID::Coal => Cow::Borrowed(
				"https://static.wikia.nocookie.net/minecraft/images/a/a7/Coal.png/revision/latest/scale-to-width/360?cb=20200814153155",
			),
			TextureID::RawIron => Cow::Borrowed(
				"https://static.wikia.nocookie.net/minecraft_gamepedia/images/d/d2/Raw_Iron_JE3_BE2.png/revision/latest?cb=20210421181435",
			),
		}
	}
}

pub fn all_textures() -> impl Iterator<Item = TextureID> {
	// &[
	// 	TextureID::Stone,
	// 	TextureID::IronOre,
	// 	TextureID::CoalOre,
	// 	TextureID::RawIron,
	// ]
	TextureID::iter()
}

/// contains all the logic for storing textures
#[derive(Debug)]
pub struct Textures {
	textures: HashMap<TextureID, Texture>,
}
impl Textures {
	pub fn new<A: Assets + Send + Sync>(
		assets: &A,
		d: &mut sui::Handle,
		thread: &sui::raylib::RaylibThread,
	) -> anyhow::Result<Self> {
		let stream = Self::stream_images(assets);
		let textures = Self::from_stream(stream, d, thread)?;

		Ok(textures)
	}

	/// loads the images in parallel, and synchronously converts the images into textures as they come
	pub fn stream_images<A: asset_provider::Assets + Sync>(
		assets: &A,
	) -> impl Stream<Item = anyhow::Result<(TextureID, DynamicImage)>> {
		let mut stream = FuturesUnordered::new();

		let resources = all_textures().map(|a| {
			let resource = a.resource_path();
			(a, resource)
		});
		let images = resources.map(async |(tile_tex, path)| {
			assets
				.asset_image(&path)
				.await
				.map(|a| (tile_tex, a))
				.with_context(|| format!("while loading {path}"))
		});

		stream.extend(images);

		stream
	}

	/// load images into textures \
	/// this has to be run on the main thread
	pub fn from_stream<S: Stream<Item = anyhow::Result<(TextureID, DynamicImage)>> + Unpin>(
		stream: S,
		d: &mut sui::Handle,
		thread: &sui::raylib::RaylibThread,
	) -> anyhow::Result<Self> {
		let mut map = HashMap::with_capacity(stream.size_hint().0);

		let iter = futures::executor::block_on_stream(stream);
		for result in iter {
			let (tex, img) = result?;

			let texture = img.texture(d, thread)?;
			map.insert(tex, texture);
		}
		Ok(Self { textures: map })
	}

	pub fn texture_for(&self, tiletex: TextureID) -> Option<&Texture> {
		self.textures.get(&tiletex)
	}
}
