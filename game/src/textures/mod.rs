use std::{
	collections::HashMap,
	sync::{Arc, OnceLock},
};

use anyhow::Context;
use asset_provider::Assets;
use asset_provider_image::{AssetsExt, ImageExt, image::DynamicImage};
use futures::{Stream, stream::FuturesUnordered};
use sui::{Color, Details, raylib::prelude::RaylibDraw, tex::Texture};

pub mod loader;
pub use loader::{load_as_layable, load_as_scene};
mod texture_id;
pub use texture_id::*;

pub(self) static INTERNAL_CACHE: OnceLock<Textures> = OnceLock::new();

/// contains all the logic for storing textures \
/// cheap to clone, and cached after the first load
#[derive(Debug, Clone)]
pub struct Textures {
	textures: Arc<HashMap<TextureID, Texture>>,
}
impl Textures {
	pub fn new(textures: HashMap<TextureID, Texture>) -> Self {
		let textures = Arc::new(textures);
		Self { textures }
	}

	/// can only cache the first instance of Textures you call .cache() on \
	/// actually you just shouldn't have two Textures instances if you're doing any type of caching
	pub fn cache(&self) {
		let cache_copy = self.clone();
		if INTERNAL_CACHE.get().is_none() {
			tokio::spawn(async move {
				let res = INTERNAL_CACHE.set(cache_copy);
				match res {
					Ok(a) => a,
					Err(_) => {
						eprintln!(
							"failed to cache Textures; most likely another instance has beed cached before"
						);
					}
				}
			});
		}
	}

	/// loads all textures synchronously \
	/// (loads the images in parallel but converts them into textures synchronously) \
	///
	/// uncached by default
	pub fn load_from_assets<A: Assets + Send + Sync>(
		assets: &A,
		d: &mut sui::Handle,
	) -> anyhow::Result<Self> {
		let stream = Self::stream_images(assets);
		let textures = Self::from_stream(stream, d)?;

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

	/// load images into textures, blocking on the stream \
	/// this has to be run on the main thread \
	///
	/// does not cache by default
	pub fn from_stream<S: Stream<Item = anyhow::Result<(TextureID, DynamicImage)>> + Unpin>(
		stream: S,
		d: &mut sui::Handle,
	) -> anyhow::Result<Self> {
		let mut map = HashMap::with_capacity(stream.size_hint().0);

		let iter = futures::executor::block_on_stream(stream);
		for result in iter {
			let (tex, img) = result?;

			let texture = img.texture(d)?;
			map.insert(tex, texture);
		}
		Ok(Self::new(map))
	}

	pub fn texture_for(&self, tiletex: TextureID) -> Option<&Texture> {
		self.textures.get(&tiletex)
	}
	pub fn texture_for_b(&self, tiletex: &TextureID) -> Option<&Texture> {
		self.textures.get(tiletex)
	}

	pub fn render(&self, d: &mut sui::Handle, det: Details, id: &TextureID) {
		const NO_TEXTURE_COLOR: Color = Color::PURPLE;

		let tex = self.texture_for_b(id);
		match tex {
			None => {
				d.draw_rectangle(det.x, det.y, det.aw, det.ah, NO_TEXTURE_COLOR);
			}
			Some(tex) => {
				tex.render(d, det);
			}
		}
	}
}
