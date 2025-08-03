use std::collections::HashMap;
use std::fmt::Debug;

use anyhow::Context;
use asset_provider::Assets;
use asset_provider_image::{ImageExt, image::DynamicImage};
use futures::StreamExt;
use stage_manager_tokio::{ConstructFunction, ConstructiveLoader};
use sui::DynamicLayable;
use sui::LayableExt;
use sui::tex::Texture;
use tokio::sync::mpsc::Sender;

use super::TextureID;
use super::Textures;

#[derive(Debug)]
enum TextureLoaderPacket {
	Finished,
	Image(anyhow::Result<(TextureID, DynamicImage)>),
}

pub fn load_as_scene<A: Assets + Send + Sync + 'static>(
	assets: A,
	post_process: impl Fn(anyhow::Result<Textures>) -> DynamicLayable<'static> + 'static,
) -> DynamicLayable<'static> {
	let loading_screen = sui::text("loading textures...", 32).centered();

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
		let (id, img) = match p {
			TextureLoaderPacket::Image(Ok(img)) => img,
			TextureLoaderPacket::Image(Err(err)) => {
				*t = Err(err);
				return true;
			}
			TextureLoaderPacket::Finished => return true,
		};

		let tex = match img.texture(d) {
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
		let textures = res.map(Textures::from_hashmap);
		post_process(textures)
	};

	let loading =
		ConstructiveLoader::new_explicit(loading_screen, f, base_t, construct, neo_post_process);
	let loading = DynamicLayable::new_only_debug(loading);

	loading
}
