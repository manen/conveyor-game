use std::{collections::HashMap, sync::Arc};

use anyhow::Context;
use asset_provider::Assets;
use futures::{StreamExt, stream::FuturesUnordered};
use game_core::maps::Tilemap;

#[derive(Clone, Debug)]
/// describes a patch of a resource
pub struct Segment {
	name: Arc<str>,
	tiles: Tilemap,
	min_distance: i32,
}
impl Segment {}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct AssetSegment {
	min_distance: i32,
}
pub type SegmentsToml = HashMap<String, AssetSegment>;

const SEGMENTS_TOML_KEY: &str = "worldgen/segments/segments.toml";
const SEGMENT_TILEMAP_KEY: &str = "worldgen/segments/{key}.cglf";

/// loads the file describing the segments
pub async fn load_segments_toml<A: Assets>(assets: A) -> anyhow::Result<SegmentsToml> {
	let asset = assets
		.asset(SEGMENTS_TOML_KEY)
		.await
		.with_context(|| format!("while reading segments.toml from assets"))?;
	let asset = asset.as_slice();

	let segments: SegmentsToml =
		toml::from_slice(asset).with_context(|| format!("while deserializing segments.toml"))?;
	Ok(segments)
}
pub async fn load_segment_tiles<A: Assets>(assets: A, name: &str) -> anyhow::Result<Tilemap> {
	let key = SEGMENT_TILEMAP_KEY.replace("{key}", name);
	let asset = assets
		.asset(&key)
		.await
		.with_context(|| format!("while loading tilemap for segment {name}"))?;
	let asset = asset.as_slice();

	let (tilemap, _) = bincode::serde::decode_from_slice(asset, bincode::config::standard())
		.with_context(|| format!("while deserializing tilemap for segment {name}"))?;
	Ok(tilemap)
}

/// loads every segment ready to be used
pub async fn load_segments<A: Assets + Clone>(assets: A) -> anyhow::Result<Vec<Segment>> {
	let segments_toml = load_segments_toml(&assets).await?;

	let segments_loader = segments_toml.into_iter().map(|(name, segment_data)| {
		let assets = assets.clone();
		async move {
			let tiles = load_segment_tiles(assets, &name)
				.await
				.with_context(|| format!("while loading every tile in segments.toml"))?;

			anyhow::Ok(Segment {
				name: name.into(),
				tiles,
				min_distance: segment_data.min_distance,
			})
		}
	});

	let mut pool = FuturesUnordered::new();
	pool.extend(segments_loader);

	let mut buf = Vec::with_capacity(pool.len());
	loop {
		let next = pool.next().await;
		match next {
			Some(Ok(a)) => buf.push(a),
			Some(Err(err)) => {
				return Err(err)
					.with_context(|| format!("while awaiting segments with FuturesUnordered"));
			}
			None => break,
		}
	}

	Ok(buf)
}
