use anyhow::Context;
use asset_provider::Assets;
use game_core::{
	ETile,
	maps::{Tilemap, TilemapExt},
};
use rand::{Rng, SeedableRng, rngs::StdRng};

use crate::{Segment, segments};

#[derive(Clone, Debug)]
pub struct WorldGenerator {
	segments: Vec<Segment>,
}
impl WorldGenerator {
	pub async fn new<A: Assets + Clone>(assets: A) -> anyhow::Result<Self> {
		let segments = segments::load_segments(assets)
			.await
			.with_context(|| format!("while creating WorldGenerator"))?;

		Ok(Self { segments })
	}

	pub fn generate_seed(&self, width: usize, height: usize, seed: u64) -> anyhow::Result<Tilemap> {
		let mut world = GeneratingWorld::new(width, height, seed);
		world.segment_in_corner(&self.segments);

		let tilemap = world.into_tilemap();
		Ok(tilemap)
	}
	pub fn generate(&self, width: usize, height: usize) -> anyhow::Result<Tilemap> {
		let mut seed = rand::rng();
		let seed: u64 = seed.random();

		self.generate_seed(width, height, seed)
	}
}

#[derive(Clone, Debug)]
pub struct GeneratingWorld<'a> {
	width: usize,
	height: usize,
	members: Vec<((i32, i32), &'a Segment)>,

	rng: StdRng,
}
impl<'a> GeneratingWorld<'a> {
	pub fn new(width: usize, height: usize, seed: u64) -> Self {
		let rng = StdRng::seed_from_u64(seed);
		Self {
			width,
			height,
			members: Default::default(),
			rng,
		}
	}

	pub fn random_segment<'b>(&mut self, segments: &'b [Segment]) -> Option<&'b Segment> {
		let len = segments.len();
		let i = self.rng.random_range(0..len);
		segments.iter().nth(i)
	}
	pub fn segment_in_corner(&mut self, segments: &'a [Segment]) -> Option<()> {
		let segment = self.random_segment(segments)?;

		let (w, h) = segment.tiles.size();
		let (w, h) = (w as i32, h as i32);

		let corner = self.rng.random_range(0..=3);
		let start_pos = match corner {
			0 => {
				// top left
				let (x, y) = (w / 2 + 1, h / 2 + 1);
				(x, y)
			}
			1 => {
				// top right
				let (x, y) = (self.width as i32 - w / 2 - 1, h / 2 + 1);
				(x, y)
			}
			2 => {
				// bottom right
				let (x, y) = (
					self.width as i32 - w / 2 - 1,
					self.height as i32 - h / 2 - 1,
				);
				(x, y)
			}
			3 => {
				// bottom left
				let (x, y) = (w / 2 + 1, self.height as i32 - h / 2 - 1);
				(x, y)
			}
			_ => {
				eprintln!("world generator generated a number it shouldn't have");
				return None;
			}
		};

		self.members.push((start_pos, segment));
		Some(())
	}

	pub fn into_tilemap(&self) -> Tilemap {
		let mut tilemap = Tilemap::stone(self.width, self.height);
		for (center_pos, segment) in &self.members {
			let width = segment.tiles.width() as i32;
			let height = segment.tiles.height() as i32;

			let start_pos_x = center_pos.0 - width / 2;
			let start_pos_y = center_pos.1 - height / 2;

			for segment_x in 0..width {
				for segment_y in 0..height {
					let segment_pos = (segment_x, segment_y);
					let world_pos = (segment_x + start_pos_x, segment_y + start_pos_y);

					// skip the stones
					match segment.tiles.at(segment_pos) {
						Some(ETile::Stone(_)) => {}
						Some(tile) => {
							let tile = tile.clone();
							if let Some(world_tile) = tilemap.at_mut(world_pos) {
								*world_tile = tile;
							}
						}
						_ => {}
					};
				}
			}
		}
		tilemap
	}
}
