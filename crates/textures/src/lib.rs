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
use tokio::sync::Mutex;

pub(self) static INTERNAL_CACHE_STATUS: OnceLock<()> = OnceLock::new();
pub(self) static INTERNAL_CACHE: Mutex<Option<Textures>> = Mutex::const_new(None);
pub fn is_cached() -> bool {
	// INTERNAL_CACHE.get().is_some()
	match INTERNAL_CACHE.try_lock() {
		Ok(a) => a.is_some(),
		_ => false,
	}
}
pub async fn clear_cache() -> Option<Textures> {
	let mut handle = INTERNAL_CACHE.lock().await;
	handle.take()
}

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
		tokio::spawn(async move {
			let mut res = INTERNAL_CACHE.lock().await;
			*res = Some(cache_copy);

			let _ = INTERNAL_CACHE_STATUS.set(());
		});
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
	) -> impl Stream<Item = anyhow::Result<(TextureID, (Vec<u8>, (i32, i32)))>> {
		let mut stream = FuturesUnordered::new();

		let resources = all_textures().map(|a| {
			let resource = a.resource_path();
			(a, resource)
		});
		let images = resources.map(async |(tex_id, path)| {
			// load asset, load into DynamicImage, load into Rgba8 then yield

			let asset = assets.asset(path.as_ref()).await?;
			let asset = asset.to_vec();

			let image = tokio::task::spawn_blocking(move || {
				let image = asset_provider_image::image::load_from_memory(&asset)?;
				let image = image.to_rgba8();

				let width = image.width() as i32;
				let height = image.height() as i32;
				let pixels = image.into_vec();
				anyhow::Ok((pixels, (width, height)))
			})
			.await??;

			anyhow::Ok((tex_id, image))
		});

		stream.extend(images);

		stream
	}

	/// load images into textures, blocking on the stream \
	/// this has to be run on the main thread \
	///
	/// does not cache by default
	pub fn from_stream<
		S: Stream<Item = anyhow::Result<(TextureID, (Vec<u8>, (i32, i32)))>> + Unpin,
	>(
		stream: S,
		d: &mut sui::Handle,
	) -> anyhow::Result<Self> {
		let mut map = HashMap::with_capacity(stream.size_hint().0);

		let iter = futures::executor::block_on_stream(stream);
		for result in iter {
			let (tex, (pixels, size)) = result?;

			let texture = Texture::new_from_rgba8(pixels, size, d)
				.with_context(|| format!("failed to load texture {tex:?}"))?;
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
