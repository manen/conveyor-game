use std::time::{Duration, Instant};

use crate::{EResource, buildings::Building};
use textures::TextureID;
use utils::Direction;

// raw resource -> (output resource, smelt duration)
fn smelt(resource: &EResource) -> Option<(EResource, Duration)> {
	match resource {
		EResource::RawIron(_) => Some((EResource::iron(), Duration::from_millis(1500))),
		_ => None,
	}
}
fn fuel(resource: &EResource) -> Option<Duration> {
	match resource {
		EResource::Coal(_) => Some(Duration::from_millis(2500)),
		_ => None,
	}
}

// --

pub const MAX_FUEL: Duration = Duration::from_secs(10);

#[derive(Copy, Clone, Debug)]
enum StartSmeltingError {
	AlreadySmelting,
	NotEnoughFuel,
	NonSmeltable,
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, Default)]
pub struct Smelter {
	fuel: Duration,

	#[serde(default)]
	resource_queue: heapless::Deque<EResource, 2>,
	#[serde(skip, default)]
	smelting: Option<SmeltData>,
}
impl Smelter {
	fn resources_free(&self, resource: &EResource) -> i32 {
		let duration = match smelt(resource) {
			Some(a) => a.1,
			None => return 0,
		};

		let smelting_free = self.smelting.is_none() as i32;
		let smelting_free = if duration <= self.fuel {
			smelting_free
		} else {
			0
		};

		let queue_free = self.resource_queue.capacity() as i32 - self.resource_queue.len() as i32;

		smelting_free + queue_free
	}

	fn start_smelting(&mut self, resource: EResource) -> Result<(), StartSmeltingError> {
		if self.smelting.is_some() {
			return Err(StartSmeltingError::AlreadySmelting);
		}

		let (out_resource, smelt_duration) =
			smelt(&resource).ok_or(StartSmeltingError::NonSmeltable)?;
		if smelt_duration > self.fuel {
			return Err(StartSmeltingError::NotEnoughFuel);
		}

		let smelt_data = SmeltData {
			end_time: Instant::now() + smelt_duration,
			output_resource: out_resource,
		};

		self.fuel -= smelt_duration;
		self.smelting = Some(smelt_data);

		Ok(())
	}
	fn receive_resource(&mut self, resource: EResource) -> Result<(), ()> {
		if self.smelting.is_none() {
			match self.start_smelting(resource.clone()) {
				Err(StartSmeltingError::NotEnoughFuel) => {}
				_ => return Ok(()),
			}
		}

		let res = self.resource_queue.push_back(resource);
		res.map_err(|_| ())
	}
}

impl Building for Smelter {
	fn name(&self) -> std::borrow::Cow<'static, str> {
		"smelter".into()
	}
	fn texture_id(&self) -> TextureID {
		TextureID::Furnace
	}

	fn render<'a>(
		&'a self,
		textures: &'a textures::Textures,
	) -> impl sui::Layable + Clone + std::fmt::Debug + 'a {
		let actively_smelting = match &self.smelting {
			Some(smelt_data) => smelt_data.end_time > Instant::now(),
			None => false,
		};
		let tid = if actively_smelting {
			TextureID::FurnaceOn
		} else {
			TextureID::Furnace
		};

		textures.texture_for(tid).cloned()
	}

	fn can_receive(&self, _from: Option<Direction>) -> bool {
		self.smelting.is_none() || self.fuel < MAX_FUEL
	}
	fn capacity_for(&self, resource: &EResource, _from: Option<Direction>) -> i32 {
		let is_smeltable = smelt(resource).is_some();
		if is_smeltable {
			let resources_free = self.resources_free(resource);
			return resources_free;
		}

		let fuel = fuel(resource);
		match fuel {
			Some(fuel) => {
				let current_fuel = self.fuel.as_millis() as i64;
				let fuel_power_millis = fuel.as_millis() as i64;
				let max_fuel_millis = MAX_FUEL.as_millis() as i64;

				let remaining = max_fuel_millis - current_fuel;
				let could_take = remaining / fuel_power_millis;

				let could_take = could_take as i32;
				could_take.max(0)
			}

			_ => 0,
		}
	}
	fn receive(&mut self, resource: EResource, _from: Option<Direction>) {
		if smelt(&resource).is_some() && self.resources_free(&resource) > 0 {
			let _ = self.receive_resource(resource);
			return;
		}

		let fuel = fuel(&resource);
		if let Some(fuel_add) = fuel {
			self.fuel += fuel_add;
			self.fuel = self.fuel.min(MAX_FUEL);
		}
	}

	fn resource_sample(
		&self,
		_tile_resource: Option<EResource>,
		_to: Option<Direction>,
	) -> Option<EResource> {
		match &self.smelting {
			Some(smelt_data) => {
				if Instant::now() >= smelt_data.end_time {
					Some(smelt_data.output_resource.clone())
				} else {
					None
				}
			}
			None => None,
		}
	}
	fn needs_poll(&self) -> bool {
		match &self.smelting {
			Some(smelt_data) => Instant::now() >= smelt_data.end_time,
			None => false,
		}
	}
	fn poll_resource(
		&mut self,
		_tile_resource: Option<EResource>,
		_to: Option<Direction>,
	) -> Option<EResource> {
		let res = match &self.smelting {
			Some(smelt_data) => {
				if Instant::now() >= smelt_data.end_time {
					Some(smelt_data.output_resource.clone())
				} else {
					None
				}
			}
			_ => None,
		};
		self.smelting.take();
		res
	}
}

#[derive(Clone, Debug)]
pub struct SmeltData {
	end_time: Instant,
	output_resource: EResource,
}
