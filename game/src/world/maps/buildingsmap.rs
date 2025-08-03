use std::{collections::HashMap, fmt::Debug};

use sui::Layable;

use crate::{
	textures::Textures,
	utils::MultiMap,
	world::{
		EResource, Map,
		buildings::{Building, EBuilding},
		maps::SIZE,
		render::{self, TILE_RENDER_SIZE},
	},
};

#[derive(Clone, Debug)]
/// BuildingsMap isn't just a type and an Ext type, but can be taken with [Self::take]
pub struct BuildingsMap {
	buildings: Map<EBuilding>,
	/// HashMap<target_position, Vec<source positions>>
	moves_queue: HashMap<(i32, i32), Vec<(i32, i32)>>,
}
impl BuildingsMap {
	pub fn new(width: usize, height: usize) -> Self {
		Self::from_map(Map::new_default(width, height))
	}
	pub fn from_map(map: Map<EBuilding>) -> Self {
		Self {
			buildings: map,
			moves_queue: HashMap::new(),
		}
	}

	pub fn width(&self) -> usize {
		self.buildings.width()
	}
	pub fn height(&self) -> usize {
		self.buildings.height()
	}

	pub fn tick(
		&mut self,
		mut tile_resource_at: impl FnMut((i32, i32)) -> Option<EResource>,
	) -> () {
		let mut tile_resource_at = |(x, y)| tile_resource_at((x as _, y as _));

		// warning: self.moves_queue gets taken as moves_queue and put back into self.moves_queue at the end of this function
		let mut moves_queue = std::mem::take(&mut self.moves_queue);

		// poll the resources from the source and push them into the target block
		let moves_total = moves_queue.multimap_drain_total();
		for (target_pos, source_pos) in moves_total {
			let mut f = || {
				let target = self.at(*target_pos)?;
				let source = self.at(source_pos)?;

				let tile_resource = tile_resource_at(source_pos);
				let sample = source.resource_sample(tile_resource.clone())?;
				let capacity = target.capacity_for(&sample);

				if capacity > 0 {
					let source = self.at_mut(source_pos)?;
					let resource = source.poll_resource(tile_resource)?;

					let target = self.at_mut(*target_pos)?;
					target.receive(resource);
				}

				Some(0)
			};
			match f() {
				Some(_) => {}
				None => {}
			}
		}

		// check for buildings that need polling and list
		for (source_pos, building) in self.iter() {
			if !building.needs_poll() {
				continue;
			}

			let relatives = building.pass_relatives();
			let target_poss = relatives
				.iter()
				.cloned()
				.map(|(rx, ry)| (source_pos.0 + rx, source_pos.1 + ry));

			for target_pos in target_poss {
				if self
					.at(target_pos)
					.map(|target| target.can_receive())
					.unwrap_or(false)
				{
					moves_queue.multimap_insert(target_pos, source_pos);
				}
			}
		}

		// sort incoming resources by the target building's preferences
		for (target_pos, source_poss) in moves_queue.iter_mut().filter(|(_, v)| !v.is_empty()) {
			let mut f = || {
				let target = self.at(*target_pos)?;

				source_poss.sort_by_key(|source_pos| {
					let rel_pos = (target_pos.0 - source_pos.0, target_pos.1 - source_pos.1);
					-target.rank_pass_source(rel_pos)
				});
				Some(1)
			};
			match f() {
				Some(_) => (),
				None => source_poss.clear(),
			}
		}
		self.moves_queue = moves_queue;
	}

	pub fn at(&self, pos: (i32, i32)) -> Option<&EBuilding> {
		self.buildings.at(pos)
	}
	pub fn at_mut(&mut self, pos: (i32, i32)) -> Option<&mut EBuilding> {
		self.buildings.at_mut(pos)
	}

	pub fn iter<'a>(&'a self) -> impl Iterator<Item = ((i32, i32), &'a EBuilding)> + 'a {
		self.buildings.iter()
	}

	pub fn take(self) -> Map<EBuilding> {
		self.buildings
	}

	pub fn render<'a, 'b: 'a>(&'a self, textures: &'b Textures) -> BuildingsRenderer<'a, 'b> {
		BuildingsRenderer::new(self, textures)
	}
}

#[derive(Clone, Debug)]
pub struct BuildingsRenderer<'a, 'b> {
	textures: &'b Textures,
	buildings: &'a BuildingsMap,
}
impl<'a, 'b> BuildingsRenderer<'a, 'b> {
	pub fn new(buildings: &'a BuildingsMap, textures: &'b Textures) -> Self {
		Self {
			textures,
			buildings,
		}
	}
}
impl<'a, 'b> Layable for BuildingsRenderer<'a, 'b> {
	fn size(&self) -> (i32, i32) {
		let size = SIZE as i32 * TILE_RENDER_SIZE as i32;
		(size, size)
	}
	fn render(&self, d: &mut sui::Handle, det: sui::Details, scale: f32) {
		render::draw_buildings(d, &self.buildings, &self.textures, det.x, det.y, scale)
	}
}
