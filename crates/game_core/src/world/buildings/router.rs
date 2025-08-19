use std::cmp::Ordering;

use crate::{
	EResource,
	buildings::{Building, CONVEYOR_CAPACITY},
};
use textures::TextureID;
use utils::Direction;

pub const ROUTER_CAPACITY: usize = CONVEYOR_CAPACITY * 2;

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, Default)]
pub struct Router {
	// (received_from, resource)
	holding: heapless::Deque<(Direction, EResource), ROUTER_CAPACITY>,

	#[serde(default)]
	pass_dir: Direction,
}
impl Router {
	/// returns index in self.holding
	fn get_available_i_for(&self, to: Direction) -> Option<usize> {
		let mut selected_i = None;
		for i in 0..self.holding.len() {
			match self.holding.iter().nth(i) {
				Some((dir, _)) => {
					if *dir == to {
						// we won't pass to where we got the resource
						continue;
					} else {
						selected_i = Some(i);
						break;
					}
				}
				None => continue,
			}
		}
		selected_i
	}

	fn count(&mut self) -> Direction {
		let dir = self.pass_dir;
		self.pass_dir = self.pass_dir.rotate_r();
		dir
	}
	fn unique_directions_in_holding(&self) -> usize {
		let mut directions: heapless::Vec<Direction, 4> = Default::default();

		for (dir, _) in self.holding.iter() {
			if directions.contains(dir) {
				continue;
			}
			let _ = directions.push(*dir);
		}
		directions.len()
	}
}
impl Building for Router {
	fn name(&self) -> std::borrow::Cow<'static, str> {
		"router".into()
	}
	fn texture_id(&self) -> TextureID {
		TextureID::Router
	}

	fn can_receive(&self, from: Option<Direction>) -> bool {
		if from.is_none() {
			return false;
		}
		self.holding.len() < ROUTER_CAPACITY
	}
	fn capacity_for(&self, _resource: &EResource, from: Option<Direction>) -> i32 {
		if from.is_none() {
			return 0;
		}
		ROUTER_CAPACITY as i32 - self.holding.len() as i32
	}
	fn receive(&mut self, resource: EResource, from: Option<Direction>) {
		match from {
			Some(from) => {
				let _ = self.holding.push_back((from, resource));
			}
			None => {}
		}
	}

	fn needs_poll(&self) -> bool {
		!self.holding.is_empty()
	}
	fn resource_sample(
		&self,
		_tile_resource: Option<EResource>,
		to: Option<Direction>,
	) -> Option<EResource> {
		let to = to?;
		let selected_i = self.get_available_i_for(to)?;

		let (_, res) = self.holding.iter().nth(selected_i)?;
		Some(res.clone())
	}
	fn poll_resource(
		&mut self,
		_tile_resource: Option<EResource>,
		to: Option<Direction>,
	) -> Option<EResource> {
		let to = to?;
		let selected_i = self.get_available_i_for(to)?;

		let (_, res) = self.holding.swap_remove_back(selected_i)?;
		Some(res)
	}

	// fn pass_relatives(&mut self) -> &'static [(i32, i32)] {
	// 	let array = self.pass_to.rel_array();
	// 	self.pass_to = self.pass_to.rotate_r();
	// 	array
	// }
	fn confirm_pass_relatives(
		&mut self,
		available_directions: &[(i32, i32)],
	) -> heapless::Vec<(i32, i32), 4> {
		if available_directions.is_empty() {
			return heapless::Vec::new();
		}

		// // available directions index (the i we went up to last time)
		// let mut last_i = 0;
		// let next_included = || {
		// 	let mut remaining = available_directions
		// 		.into_iter()
		// 		.copied()
		// 		.enumerate()
		// 		.skip(last_i);

		// 	// this isn't likely to go infinite cause self.count cycles over Direction so 3 cycles at max
		// 	// unless available_directions is empty
		// 	loop {
		// 		if last_i >= available_directions.len() - 1 {
		// 			break None;
		// 		}

		// 		let dir = self.count();
		// 		let rel = dir.rel();
		// 		if let Some((i, _)) = remaining.find(|(_, available_rel)| rel == *available_rel) {
		// 			last_i = i;
		// 			break Some(rel);
		// 		}
		// 	}
		// };

		available_directions.iter().cloned().collect()
	}

	fn pass_relatives(&self) -> heapless::Vec<(i32, i32), 4> {
		let unique_in_holding = self.unique_directions_in_holding();
		let to_exclude_dir = match unique_in_holding {
			0 => return heapless::Vec::new(),
			1 => {
				let (to_exclude_dir, _) = self.holding.iter().next()
					.expect("if there's at least 1 unique direction in holding then there's at least one element in holding");
				Some(*to_exclude_dir)
			}
			1.. => None,
		};

		Direction::all()
			.filter(|dir| Some(*dir) != to_exclude_dir)
			.map(|dir| dir.rel())
			.collect()
	}
}
