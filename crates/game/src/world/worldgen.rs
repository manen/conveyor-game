use std::{
	rc::Rc,
	time::{Duration, SystemTime},
};

use fastnoise_lite::{FastNoiseLite, NoiseType};

use crate::world::ETile;

pub fn gen_tiles_from_seed_iter(
	seed: i32,
	width: usize,
	height: usize,
) -> impl Iterator<Item = impl Iterator<Item = ETile>> {
	let mut noise = FastNoiseLite::new();
	noise.seed = seed;
	noise.set_noise_type(Some(NoiseType::Perlin));
	let noise = Rc::new(noise);

	let f = move |x, y| {
		let noise = noise.get_noise_2d(x as f32, y as f32);
		println!("{noise:?}");

		let noise = noise * 10000000.0;

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
