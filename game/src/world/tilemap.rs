use std::collections::HashMap;

use anyhow::Context;
use asset_provider::Assets;
use futures::{Stream, stream::FuturesUnordered};
use sui::tex::Texture;

use asset_provider_image::{AssetsExt, ImageExt, image::DynamicImage};

use crate::world::{
	STile,
	tile::{TileTexture, tiles::Stone, tiletex::all_textures},
};

/// world size in tiles
pub const SIZE: usize = 32;
pub type TileTextures = HashMap<TileTexture, Texture>;

#[derive(Clone, Debug)]
pub struct Tilemap {
	textures: TileTextures,
	tiles: [[STile; SIZE]; SIZE],
}
impl Tilemap {
	pub fn new<A: Assets + Send + Sync>(
		assets: &A,
		d: &mut sui::Handle,
		thread: &sui::raylib::RaylibThread,
	) -> anyhow::Result<Self> {
		let stream = Self::stream_images(assets);
		let textures = Self::load_textures_from_stream(stream, d, thread)?;

		let game = Self::from_textures(textures)?;
		Ok(game)
	}

	/// load images into textures \
	/// this has to be run on the main thread:
	pub fn load_textures_from_stream<
		S: Stream<Item = anyhow::Result<(TileTexture, DynamicImage)>> + Unpin,
	>(
		stream: S,
		d: &mut sui::Handle,
		thread: &sui::raylib::RaylibThread,
	) -> anyhow::Result<TileTextures> {
		let mut map = HashMap::with_capacity(stream.size_hint().0);

		let iter = futures::executor::block_on_stream(stream);
		for result in iter {
			let (tex, img) = result?;

			let texture = img.texture(d, thread)?;
			map.insert(tex, texture);
		}
		Ok(map)
	}

	/// loads the images in parallel, and synchronously converts the images into textures as they come
	pub fn stream_images<A: asset_provider::Assets + Sync>(
		assets: &A,
	) -> impl Stream<Item = anyhow::Result<(TileTexture, DynamicImage)>> {
		let mut stream = FuturesUnordered::new();

		let resources = all_textures().iter().cloned().map(|a| {
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

	pub fn gen_tiles() -> [[STile; SIZE]; SIZE] {
		core::array::from_fn(|x| core::array::from_fn(|y| STile::Stone(Stone)))
	}
	/// fetch textures with [Self::load_textures] \
	/// [Self::load_textures] needs to be run on the main thread!
	pub fn from_textures(
		// from [Self::load_textures]
		textures: HashMap<TileTexture, Texture>,
	) -> anyhow::Result<Self> {
		let tiles = Self::gen_tiles();
		Ok(Self { textures, tiles })
	}

	pub fn at(&self, (x, y): (usize, usize)) -> Option<&STile> {
		if x > SIZE - 1 {
			if x > SIZE - 1 {
				return None;
			}
		}

		Some(&self.tiles[x][y])
	}
	pub fn texture_for(&self, tiletex: TileTexture) -> Option<&Texture> {
		self.textures.get(&tiletex)
	}
}
