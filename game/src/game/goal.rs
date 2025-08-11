use std::collections::HashMap;

use tokio::sync::mpsc::{self, error::TryRecvError};

use crate::world::EResource;

#[derive(Debug)]
/// defines what the goal is for the level
pub struct Goal {
	resources: HashMap<EResource, i32>,
}
impl Goal {
	pub fn new<I: IntoIterator<Item = (EResource, i32)>>(iter: I) -> Self {
		Self {
			resources: iter.into_iter().collect(),
		}
	}
	pub fn from_hashmap(resources: HashMap<EResource, i32>) -> Self {
		Self { resources }
	}
}

#[derive(Debug)]
/// the wrapper for the resource receiver to make checking for the goal easier
pub struct ResourceCounter {
	goal: Goal,

	resource_rx: mpsc::Receiver<EResource>,
	resources: HashMap<EResource, i32>,
}
impl ResourceCounter {
	pub fn new(goal: Goal, resource_rx: mpsc::Receiver<EResource>) -> Self {
		Self {
			goal,
			resource_rx,
			resources: Default::default(),
		}
	}

	fn tick(&mut self) {
		loop {
			match self.resource_rx.try_recv() {
				Ok(resource) => {
					// add the received resource to the counter
					let count = self.resources.get(&resource).copied().unwrap_or_default();
					self.resources.insert(resource, count);
				}
				Err(TryRecvError::Empty) => break,
				Err(TryRecvError::Disconnected) => {
					eprintln!(
						"ResourceCounter's receiver disconnected: ChannelConsumer buildings destroyed or game got dropped"
					);
					break;
				}
			}
		}
	}

	pub fn set_goal(&mut self, goal: Goal) {
		self.goal = goal;
	}

	pub fn is_reached(&mut self) -> bool {
		self.tick();
		self.is_reached_no_tick()
	}
	fn is_reached_no_tick(&self) -> bool {
		self.goal
			.resources
			.iter()
			.filter(|(resource, required_count)| {
				let count = self.resources.get(resource).copied().unwrap_or_default();

				**required_count < count
			})
			.count() > 0
	}
}

// ---

use crate::textures::Textures;

/// proper goal & progress rendering would need both the received resources,

#[derive(Clone, Debug)]
pub struct ResourcesRenderer<'a> {
	textures: &'a Textures,
	resources: &'a HashMap<EResource, i32>,
}
impl<'a> ResourcesRenderer<'a> {
	pub fn new(textures: &'a Textures, resources: &'a HashMap<EResource, i32>) -> Self {
		Self {
			textures,
			resources,
		}
	}
}
