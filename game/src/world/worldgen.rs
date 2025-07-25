use std::time::{Duration, SystemTime};

use perlin2d::PerlinNoise2D;

use crate::world::{ETile, tilemap::SIZE};

pub fn gen_tiles_from_seed(seed: i32) -> [[ETile; SIZE]; SIZE] {
	let perlin = PerlinNoise2D::new(6, 10.0, 0.5, 1.0, 2.0, (20.0, 20.0), 0.5, seed);

	core::array::from_fn(|x| {
		core::array::from_fn(|y| {
			let noise = perlin.get_noise(x as f64, y as f64);

			let noise_adj = noise % 8.0;
			match noise_adj {
				1.0..2.0 => ETile::coal_ore(),
				3.0..5.0 => ETile::iron_ore(),
				_ => ETile::stone(),
			}
		})
	})
}

pub fn gen_tiles() -> [[ETile; SIZE]; SIZE] {
	// let mut rep = std::iter::repeat([
	// 	STile::Stone(tiles::Stone),
	// 	STile::IronOre(tiles::IronOre),
	// 	STile::CoalOre(tiles::CoalOre),
	// ])
	// .flatten();

	// core::array::from_fn(|x| core::array::from_fn(|y| rep.next().unwrap()))

	let start = SystemTime::UNIX_EPOCH + Duration::from_secs(1420070400); // 2015-01-01T00:00:00Z (or so i'm told by chatgpt)
	let now = SystemTime::now();
	let elapsed = now.duration_since(start).unwrap();

	gen_tiles_from_seed((elapsed.as_millis() % i32::MAX as u128) as i32)
}
