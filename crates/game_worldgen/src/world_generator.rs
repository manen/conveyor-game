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

		world.segments_in_corner(&self.segments, 2);

		world.segment_at_farthest(&self.segments);
		world.segment_at_farthest(&self.segments);
		world.segment_at_farthest(&self.segments);
		world.segment_at_farthest(&self.segments);
		world.segment_at_farthest(&self.segments);

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
	pub fn segments_in_corner(&mut self, segments: &'a [Segment], count: i32) -> Option<()> {
		let counter = std::iter::repeat([0, 1, 2, 3]).flatten();
		let offset = self.rng.random_range(0..=3);
		let mut counter = counter.skip(offset as _);

		for _ in 0..count {
			let segment = self.random_segment(segments)?;
			let (w, h) = segment.tiles.size();
			let (w, h) = (w as i32, h as i32);

			let corner = counter.next()?;
			mklogger::println!("chose corner {corner}");
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
					mklogger::eprintln!("world generator generated a number it shouldn't have");
					return None;
				}
			};

			self.members.push((start_pos, segment));
		}
		Some(())
	}

	/// returns the coordinate farthest from any segment \
	/// returns (distance from closest segment, coords)
	pub fn farthest_coordinate(&self) -> (f32, (i32, i32)) {
		if self.members.len() <= 0 {
			return (0.0, (0, 0));
		}

		let mut highest_yet = (0.0, (0, 0));

		// i'd rather not calculate the big o notation for this
		for x in 0..(self.width as i32) {
			for y in 0..(self.height as i32) {
				let mut local_lowest = f32::MAX;

				for (segment_pos, _) in self.members.iter() {
					let dst = dst((x, y), *segment_pos);
					local_lowest = local_lowest.min(dst);
				}

				if local_lowest > highest_yet.0 {
					highest_yet = (local_lowest, (x, y))
				}
			}
		}

		highest_yet
	}
	pub fn farthers_to_fit(&self, s_width: usize, s_height: usize) -> (i32, i32) {
		let (_, (x, y)) = self.farthest_coordinate();

		let smallest_x = s_width as i32 / 2 + 1;
		let smallest_y = s_height as i32 / 2 + 1;
		let biggest_x = self.width as i32 - smallest_x;
		let biggest_y = self.height as i32 - smallest_y;

		let x = x.max(smallest_x).min(biggest_x);
		let y = y.max(smallest_y).min(biggest_y);

		(x, y)
	}
	pub fn segment_at_farthest(&mut self, segments: &'a [Segment]) -> Option<()> {
		let segment = self.random_segment(segments)?;
		let (w, h) = segment.tiles.size();

		let farthest = self.farthers_to_fit(w, h);
		mklogger::println!("farthest: {farthest:?}");

		self.members.push((farthest, segment));

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

fn dst((a_x, a_y): (i32, i32), (b_x, b_y): (i32, i32)) -> f32 {
	let x_diff = (a_x - b_x) as f32;
	let y_diff = (a_y - b_y) as f32;

	(x_diff * x_diff + y_diff * y_diff).sqrt()
}
