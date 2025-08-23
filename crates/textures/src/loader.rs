use std::collections::HashMap;
use std::fmt::Debug;

use anyhow::Context;
use asset_provider::Assets;
use asset_provider_image::{ImageExt, image::DynamicImage};
use futures::StreamExt;
use stage_manager::StageChange;
use stage_manager_loaders::{ConstructFunction, ConstructiveLoader};
use sui::DynamicLayable;
use sui::Layable;
use sui::LayableExt;
use sui::tex::Texture;
use tokio::sync::mpsc::Sender;

use super::TextureID;
use super::Textures;

use super::INTERNAL_CACHE;

#[derive(Debug)]
pub enum TextureLoaderPacket {
	Image(anyhow::Result<(TextureID, (Vec<u8>, (i32, i32)))>),
	Finished,
}

pub fn loading_screen() -> impl Layable + Debug + Clone {
	sui::text("loading textures...", 32)
		.with_background(sui::comp::Color::new(sui::Color::BLACK))
		.centered()
}

mod basic {
	use super::*;

	pub fn load_as_layable_explicit<A: Assets + Send + Sync + 'static>(
		assets: A,
		loading_screen: DynamicLayable<'static>,
		post_process: impl FnOnce(anyhow::Result<Textures>) -> DynamicLayable<'static> + 'static,
	) -> DynamicLayable<'static>
// ConstructiveLoader<
		// 	anyhow::Result<HashMap<TextureID, Texture>>,
		// 	TextureLoaderPacket,
		// 	impl Fn(anyhow::Result<HashMap<TextureID, Texture>>) -> StageChange<'static>, // one of the function signatures of all time
		// >
	{
		let f = async move |tx: Sender<_>| {
			let mut stream = Textures::stream_images(&assets);
			loop {
				let img = stream.next().await;
				if let Some(img) = img {
					tx.send(TextureLoaderPacket::Image(img)).await.unwrap();
				} else {
					tx.send(TextureLoaderPacket::Finished).await.unwrap();
					break;
				}
			}
		};

		let base_t = anyhow::Ok(HashMap::new());

		fn construct(
			t: &mut anyhow::Result<HashMap<TextureID, Texture>>,
			p: TextureLoaderPacket,
			d: &mut sui::Handle,
		) -> bool {
			let (id, (pixels, size)) = match p {
				TextureLoaderPacket::Image(Ok(img)) => img,
				TextureLoaderPacket::Image(Err(err)) => {
					*t = Err(err);
					return true;
				}
				TextureLoaderPacket::Finished => return true,
			};

			let tex = match Texture::new_from_rgba8(pixels, size, d) {
				Ok(a) => a,
				Err(err) => {
					*t = Err(err)
						.with_context(|| format!("while converting a loaded image into a texture"));
					return true;
				}
			};

			match t {
				Ok(textures) => {
					textures.insert(id, tex);
					false
				}
				Err(_) => true,
			}
		}
		let construct = ConstructFunction::NeedsSuiHandle(construct);

		let neo_post_process = move |res: Result<HashMap<TextureID, Texture>, anyhow::Error>| {
			let textures = res.map(Textures::new);
			stage_manager::StageChange::Simple(post_process(textures))
		};

		let loader = ConstructiveLoader::new_explicit(
			loading_screen,
			f,
			base_t,
			construct,
			neo_post_process,
		);
		sui::custom_only_debug(loader)
	}
}

mod cached {
	use super::*;

	pub fn load_as_layable_explicit<A: Assets + Send + Sync + 'static>(
		assets: A,
		loading_screen: DynamicLayable<'static>,
		post_process: impl FnOnce(anyhow::Result<Textures>) -> DynamicLayable<'static> + 'static,
	) -> DynamicLayable<'static> {
		match INTERNAL_CACHE.try_lock() {
			Ok(guard) => {
				match guard.as_ref() {
					Some(cached) => {
						// when cached, we skip all the loader steps and jump right to the final layable

						let cached = cached.clone();
						let processed = post_process(Ok(cached));
						return processed;
					}
					None => {}
				}
			}
			Err(_) => {}
		}

		let cache_and_post_process = move |res: anyhow::Result<Textures>| match res {
			Ok(textures) => {
				textures.cache();
				post_process(Ok(textures))
			}
			_ => post_process(res),
		};

		let loader =
			basic::load_as_layable_explicit(assets, loading_screen, cache_and_post_process);
		loader
	}
}
pub use cached::load_as_layable_explicit;

/// same as load_as_scene, but load_as_scene return a StageChange (it'll keep the old stage in the background while it's loading)
pub fn load_as_layable<A: Assets + Send + Sync + 'static>(
	assets: A,
	post_process: impl Fn(anyhow::Result<Textures>) -> DynamicLayable<'static> + 'static,
) -> DynamicLayable<'static> {
	let loading_screen = sui::custom(loading_screen());
	let loader = load_as_layable_explicit(assets, loading_screen, post_process);

	sui::custom_only_debug(loader)
}

pub fn load_as_scene<A: Assets + Send + Sync + 'static>(
	assets: A,
	post_process: impl FnOnce(anyhow::Result<Textures>) -> DynamicLayable<'static> + 'static,
) -> StageChange<'static> {
	StageChange::swapper(move |old_stage| {
		let loading_screen = sui::custom(old_stage.overlay(loading_screen()));

		sui::custom_only_debug(load_as_layable_explicit(
			assets,
			loading_screen,
			post_process,
		))
	})
}
