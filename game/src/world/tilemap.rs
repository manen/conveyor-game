use std::{borrow::Cow, collections::HashMap};

use futures::{StreamExt, stream::FuturesUnordered};
use sui::{raylib::RaylibThread, tex::Texture};

use asset_provider_image::{AssetsExt, ImageExt, image::DynamicImage};

use crate::world::{
	STile,
	tile::{Stone, TileTexture, tiletex::all_textures},
};

pub const SIZE: usize = 32;

#[derive(Clone, Debug)]
pub struct Tilemap {
	textures: HashMap<TileTexture, Texture>,
	tiles: [[STile; SIZE]; SIZE],
}
impl Tilemap {
	pub async fn new<A: asset_provider::Assets + Sync>(
		assets: &A,
		d: &mut sui::Handle<'_>,
		thread: &RaylibThread,
	) -> anyhow::Result<Self> {
		let mut stream = FuturesUnordered::new();
		stream.extend(
			all_textures()
				.iter()
				.cloned()
				.filter_map(|a| {
					let path = a.resource_path()?;
					Some((a, path))
				})
				.map(async |(tile_tex, path)| {
					assets.asset_image(&path).await.map(|a| (tile_tex, a))
				}),
		);
		let images: Vec<Result<(TileTexture, DynamicImage), _>> = stream.collect().await;
		let images = images.into_iter().collect::<Result<Vec<_>, _>>()?;

		let textures = images
			.into_iter()
			.map(|(tile_tex, img)| img.texture(d, thread).map(|a| (tile_tex, a)))
			.collect::<Result<HashMap<_, _>, _>>()?;

		Ok(Self {
			textures,
			tiles: core::array::from_fn(|x| {
				core::array::from_fn(|y| {
					if (y + x) % 2 == 0 {
						STile::Stone(Stone)
					} else {
						Default::default()
					}
				})
			}),
		})
	}
}
