use std::sync::OnceLock;

use anyhow::{Context, anyhow};
use asset_provider::Assets;
use game_core::maps::Tilemap;

pub mod old_perlin;

pub mod segments;
pub use segments::*;

pub mod world_generator;
pub use world_generator::*;

// pub fn gen_world(width: usize, height: usize) -> Tilemap {
// 	let tiles = gen_tiles(width, height);
// 	Tilemap::from_vec(tiles).expect("gen_tiles doesn't return a correct Tilemap")
// }

static SHARED_WORLDGEN: OnceLock<WorldGenerator> = OnceLock::new();
pub async fn init_worldgen<A: Assets + Clone>(assets: A) -> anyhow::Result<()> {
	let generator = WorldGenerator::new(assets)
		.await
		.with_context(|| format!("while initializing WorldGenerator"))?;

	let res = SHARED_WORLDGEN.set(generator);
	let res = res.map_err(|worldgen| {
		anyhow!("global WorldGenerator has already been initialized\n{worldgen:?}")
	});
	res
}

pub fn gen_world(width: usize, height: usize) -> anyhow::Result<Tilemap> {
	let worldgen = SHARED_WORLDGEN
		.get()
		.ok_or_else(|| anyhow!("global WorldGenerator hasn't been initialized"))?;

	worldgen.generate(width, height)
}
