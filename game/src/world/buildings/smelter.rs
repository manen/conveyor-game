use std::time::{Duration, Instant};

use crate::{
	textures::TextureID,
	utils::Direction,
	world::{EResource, buildings::Building},
};

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

#[derive(Clone, Debug, Default)]
pub struct Smelter {
	fuel: Duration,
	smelting: Option<SmeltData>,
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
		textures: &'a crate::textures::Textures,
	) -> impl sui::Layable + Clone + std::fmt::Debug + 'a {
		let tid = if self.smelting.is_some() {
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
		let is_smelting = self.smelting.is_some(); // <- filter raw resources if we're already smelting
		let smelted = if !is_smelting { smelt(resource) } else { None };

		let fuel = fuel(resource);

		match (smelted, fuel) {
			(Some((_, smelt_duration)), _) => {
				if self.fuel >= smelt_duration {
					1
				} else {
					0
				}
			}
			(_, Some(fuel)) => {
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
		let is_smelting = self.smelting.is_some(); // <- filter raw resources if we're already smelting
		if !is_smelting {
			let smelted = smelt(&resource);
			if let Some((out_resource, smelt_duration)) = smelted {
				let smelt_data = SmeltData {
					end_time: Instant::now() + smelt_duration,
					output_resource: out_resource,
				};

				self.smelting = Some(smelt_data);
				return;
			}
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
