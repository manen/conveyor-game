use crate::{EResource, buildings::Building};
use textures::TextureID;
use utils::Direction;

use super::conveyor::CONVEYOR_CAPACITY;

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, Default)]
pub struct Junction {
	up: heapless::Deque<EResource, CONVEYOR_CAPACITY>,
	down: heapless::Deque<EResource, CONVEYOR_CAPACITY>,
	left: heapless::Deque<EResource, CONVEYOR_CAPACITY>,
	right: heapless::Deque<EResource, CONVEYOR_CAPACITY>,
}
impl Junction {
	fn queue_for(&self, dir: Direction) -> &heapless::Deque<EResource, CONVEYOR_CAPACITY> {
		match dir {
			Direction::Top => &self.up,
			Direction::Bottom => &self.down,
			Direction::Left => &self.left,
			Direction::Right => &self.right,
		}
	}
	fn queue_for_mut(
		&mut self,
		dir: Direction,
	) -> &mut heapless::Deque<EResource, CONVEYOR_CAPACITY> {
		match dir {
			Direction::Top => &mut self.up,
			Direction::Bottom => &mut self.down,
			Direction::Left => &mut self.left,
			Direction::Right => &mut self.right,
		}
	}
}
impl Building for Junction {
	fn name(&self) -> std::borrow::Cow<'static, str> {
		"junction".into()
	}
	fn texture_id(&self) -> TextureID {
		TextureID::Junction
	}

	fn can_receive(&self, from: Option<Direction>) -> bool {
		match from {
			Some(dir) => self.queue_for(dir).len() < CONVEYOR_CAPACITY,
			None => false,
		}
	}
	fn capacity_for(&self, _resource: &EResource, from: Option<Direction>) -> i32 {
		match from {
			Some(dir) => CONVEYOR_CAPACITY as i32 - self.queue_for(dir).len() as i32,
			None => 0,
		}
	}
	fn receive(&mut self, resource: EResource, from: Option<Direction>) {
		match from {
			Some(dir) => {
				let _ = self.queue_for_mut(dir).push_back(resource);
			}
			None => (),
		}
	}

	fn needs_poll(&self) -> bool {
		Direction::all()
			.filter(|dir| self.queue_for(*dir).len() > 0)
			.count() > 0
	}
	fn resource_sample(
		&self,
		_tile_resource: Option<EResource>,
		to: Option<Direction>,
	) -> Option<EResource> {
		match to {
			Some(dir) => self.queue_for(dir.reverse()).iter().cloned().next(),
			None => None,
		}
	}
	fn poll_resource(
		&mut self,
		_tile_resource: Option<EResource>,
		to: Option<Direction>,
	) -> Option<EResource> {
		match to {
			Some(dir) => self.queue_for_mut(dir.reverse()).pop_front(),
			None => None,
		}
	}
	fn pass_relatives(&self) -> &'static [(i32, i32)] {
		Direction::all_rel_array()
	}
}
