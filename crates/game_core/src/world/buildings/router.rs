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
	holding: heapless::Deque<EResource, ROUTER_CAPACITY>,

	#[serde(default)]
	pass_dir: Direction,
}
impl Router {
	fn count(&mut self) -> Direction {
		let dir = self.pass_dir;
		// println!("router cycling over {dir:?}");

		self.pass_dir = self.pass_dir.rotate_r();
		dir
	}
	fn count_included(
		&mut self,
		available_directions: impl Iterator<Item = (i32, i32)> + Clone,
	) -> Option<Direction> {
		let actual_directions = available_directions
			.clone()
			.map(Direction::from_rel)
			.filter(|a| a.is_some())
			.count();
		// println!("{actual_directions:?}");
		if actual_directions == 0 {
			return None;
		}

		for _ in 0..5 {
			let dir = self.count();
			let rel = dir.rel();

			let contained = available_directions
				.clone()
				.find(|available_rel| *available_rel == rel);
			if contained.is_some() {
				// println!("next direction is {dir:?}");
				return Some(dir);
			} else {
				// println!("{dir:?} nope");
				continue;
			}
		}
		None
	}

	// fn unique_directions_in_holding(&self) -> usize {
	// 	let mut directions: heapless::Vec<Direction, 4> = Default::default();

	// 	for (dir, _) in self.holding.iter() {
	// 		if directions.contains(dir) {
	// 			continue;
	// 		}
	// 		let _ = directions.push(*dir);
	// 	}
	// 	directions.len()
	// }
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
	fn receive(&mut self, resource: EResource, _from: Option<Direction>) {
		let _ = self.holding.push_back(resource);
	}

	fn needs_poll(&self) -> bool {
		!self.holding.is_empty()
	}
	fn resource_sample(
		&self,
		_tile_resource: Option<EResource>,
		_to: Option<Direction>,
	) -> Option<EResource> {
		self.holding.iter().next().cloned()
	}
	fn poll_resource(
		&mut self,
		_tile_resource: Option<EResource>,
		_to: Option<Direction>,
	) -> Option<EResource> {
		self.holding.pop_front()
	}

	// fn pass_relatives(&mut self) -> &'static [(i32, i32)] {
	// 	let array = self.pass_to.rel_array();
	// 	self.pass_to = self.pass_to.rotate_r();
	// 	array
	// }
	fn confirm_pass_relatives(
		&mut self,
		available_directions: impl Iterator<Item = (i32, i32)> + Clone,
	) -> heapless::Vec<(i32, i32), 4> {
		// let ad_buf = available_directions.clone().collect::<Vec<_>>();
		// println!("{ad_buf:?}");

		if available_directions.clone().count() == 0 {
			return heapless::Vec::new();
		}
		let mut buf = heapless::Vec::new();

		let pass_count = available_directions.clone().count().min(self.holding.len());
		// println!("pass_count: {pass_count}");
		for _ in 0..pass_count {
			let next_included = self.count_included(available_directions.clone());
			// dbg!(next_included);
			if let Some(dir) = next_included {
				let rel = dir.rel();
				let _ = buf.push(rel);
			}
		}
		// println!("confirmed: {buf:?}");
		buf

		// available_directions.collect()
	}

	fn pass_directions(&self) -> heapless::Vec<Direction, 4> {
		// let unique_in_holding = self.unique_directions_in_holding();
		// let to_exclude_dir = match unique_in_holding {
		// 	0 => return heapless::Vec::new(),
		// 	1 => {
		// 		let (to_exclude_dir, _) = self.holding.iter().next()
		// 			.expect("if there's at least 1 unique direction in holding then there's at least one element in holding");
		// 		Some(*to_exclude_dir)
		// 	}
		// 	1.. => None,
		// };

		// Direction::all()
		// 	.filter(|dir| Some(*dir) != to_exclude_dir)
		// 	.map(|dir| dir.rel())
		// 	.collect()

		Direction::all().collect()
	}
}
