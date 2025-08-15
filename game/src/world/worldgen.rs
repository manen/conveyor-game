use std::time::{Duration, SystemTime};

use perlin2d::PerlinNoise2D;

use crate::world::ETile;

pub fn gen_tiles_from_seed_iter(
	seed: i32,
	width: usize,
	height: usize,
) -> impl Iterator<Item = impl Iterator<Item = ETile>> {
	let perlin = PerlinNoise2D::new(6, 10.0, 0.5, 1.0, 2.0, (20.0, 20.0), 0.5, seed);
	let perlin = std::sync::Arc::new(perlin);

	let f = move |x, y| {
		let noise = perlin.get_noise(x as f64, y as f64);

		let noise_adj = noise % 8.0;
		match noise_adj {
			1.0..2.0 => ETile::coal_ore(),
			3.0..5.0 => ETile::iron_ore(),
			_ => ETile::stone(),
		}
	};

	(0..width).map(move |x| {
		let f = f.clone();
		(0..height).map(move |y| f(x, y))
	})
}

pub fn gen_tiles_iter(
	width: usize,
	height: usize,
) -> impl Iterator<Item = impl Iterator<Item = ETile>> {
	// let mut rep = std::iter::repeat([
	// 	STile::Stone(tiles::Stone),
	// 	STile::IronOre(tiles::IronOre),
	// 	STile::CoalOre(tiles::CoalOre),
	// ])
	// .flatten();

	// core::array::from_fn(|x| core::array::from_fn(|y| rep.next().unwrap()))

	let start = SystemTime::UNIX_EPOCH + Duration::from_secs(1420070400 + 19526400); // 2015-01-01T00:00:00Z (or so i'm told by chatgpt)
	let now = SystemTime::now();
	let elapsed = now.duration_since(start).unwrap();

	gen_tiles_from_seed_iter(
		(elapsed.as_millis() % i32::MAX as u128) as i32,
		width,
		height,
	)
}

pub fn gen_tiles(width: usize, height: usize) -> Vec<Vec<ETile>> {
	gen_tiles_iter(width, height).map(|a| a.collect()).collect()
}
