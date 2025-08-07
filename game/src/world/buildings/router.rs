use crate::{
	textures::TextureID,
	utils::Direction,
	world::{
		EResource,
		buildings::{Building, CONVEYOR_CAPACITY},
	},
};

pub const ROUTER_CAPACITY: usize = CONVEYOR_CAPACITY * 2;

#[derive(Clone, Debug, Default)]
pub struct Router {
	// (received_from, resource)
	holding: heapless::Deque<(Direction, EResource), ROUTER_CAPACITY>,
	pass_to: Direction,
}
impl Router {
	/// returns index in self.holding
	fn get_available_i_for(&self, to: Direction) -> Option<usize> {
		let mut selected_i = None;
		for i in 0..self.holding.len() {
			match self.holding.get(i) {
				Some((dir, _)) => {
					if *dir == to {
						// we won't pass to where we got the resource
						continue;
					} else {
						selected_i = Some(i);
					}
				}
				None => continue,
			}
		}
		selected_i
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

	fn pass_relatives(&mut self) -> &'static [(i32, i32)] {
		let array = self.pass_to.rel_array();
		self.pass_to = self.pass_to.rotate_r();
		array
	}
}
