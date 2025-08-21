mod worldgen;
use game_core::maps::Tilemap;
pub use worldgen::*;

pub fn gen_world(width: usize, height: usize) -> Tilemap {
	let tiles = gen_tiles(width, height);
	Tilemap::from_vec(tiles).expect("gen_tiles doesn't return a correct Tilemap")
}
