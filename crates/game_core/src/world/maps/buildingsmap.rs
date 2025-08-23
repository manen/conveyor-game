use std::{collections::HashMap, fmt::Debug};

use sui::Layable;

use crate::{
	EResource, Map,
	buildings::{Building, EBuilding},
	maps::SIZE,
	render::{self, TILE_RENDER_SIZE},
};
use textures::Textures;
use utils::{Direction, MultiMap};

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum OrIndexed<T> {
	Indexed { root: (i32, i32), index: usize },
	Item(T),
}

#[derive(Clone, Debug)]
/// BuildingsMap isn't just a type and an Ext type, but can be taken with [Self::take]
pub struct BuildingsMap {
	buildings_grid: Map<OrIndexed<EBuilding>>,

	/// external_buildings contains buildings, not on a grid, but indexed. \
	/// useful to share building implementations between different 1x1 grid buildings
	external_buildings: Vec<EBuilding>,

	/// HashMap<target_position, Vec<source positions>>
	moves_queue: HashMap<(i32, i32), Vec<(i32, i32)>>,
}
impl BuildingsMap {
	pub fn new_default(width: usize, height: usize) -> Self {
		Self::from_grid(Map::new_default(width, height))
	}
	pub fn from_grid(buildings_grid: Map<EBuilding>) -> Self {
		let map = buildings_grid.map(OrIndexed::Item);
		Self {
			buildings_grid: map,
			external_buildings: Default::default(),
			moves_queue: Default::default(),
		}
	}

	pub fn width(&self) -> usize {
		self.buildings_grid.width()
	}
	pub fn height(&self) -> usize {
		self.buildings_grid.height()
	}
	pub fn size(&self) -> (usize, usize) {
		self.buildings_grid.size()
	}

	pub fn tick(
		&mut self,
		mut tile_resource_at: impl FnMut((i32, i32)) -> Option<EResource>,
	) -> () {
		let mut tile_resource_at = |(x, y)| tile_resource_at((x as _, y as _));

		let mut target_poss_buf = Vec::new();

		// warning: self.moves_queue gets taken as moves_queue and put back into self.moves_queue at the end of this function
		let mut moves_queue = std::mem::take(&mut self.moves_queue);

		// execute the queue
		// poll the resources from the source and push them into the target block
		let moves_total = moves_queue.multimap_drain_total();
		for (target_pos, source_pos) in moves_total {
			let mut f = || {
				let target = self.at(*target_pos)?;
				let source = self.at(source_pos)?;

				let from =
					Direction::from_rel((source_pos.0 - target_pos.0, source_pos.1 - target_pos.1));
				let to = from.map(Direction::reverse);

				let tile_resource = tile_resource_at(source_pos);
				let sample = source.resource_sample(tile_resource.clone(), to)?;
				let capacity = target.capacity_for(&sample, from);

				if capacity > 0 {
					let source = self.at_mut(source_pos)?;
					let resource = source.poll_resource(tile_resource, to)?;

					let target = self.at_mut(*target_pos)?;
					target.receive(resource, from);
				}

				Some(0)
			};
			match f() {
				Some(_) => {}
				None => {}
			}
		}

		// check for buildings that need polling and list
		for source_pos in self.buildings_grid.iter_coords() {
			if !self
				.at(source_pos)
				.map(Building::needs_poll)
				.unwrap_or(false)
			{
				continue;
			}

			let building = self.at_mut(source_pos).unwrap();
			let relatives = building.pass_directions();

			let target_poss = relatives.iter().cloned().filter_map(|dir| {
				let (rx, ry) = dir.rel();
				let dir = Some(dir);

				let target_pos = (source_pos.0 + rx, source_pos.1 + ry);
				let can_receive = {
					let target = self.at(target_pos)?;
					target.can_receive(dir.map(Direction::reverse))
				};
				if can_receive { Some(target_pos) } else { None }
			});
			target_poss_buf.clear();
			target_poss_buf.extend(target_poss);

			let selected_target = {
				let source = self.at_mut(source_pos).expect(
					"if you check to see where source_pos is generated it's guaranteed to exist",
				);

				let available_rels = target_poss_buf
					.iter()
					.copied()
					.map(|(target_x, target_y)| (target_x - source_pos.0, target_y - source_pos.1))
					.filter_map(Direction::from_rel);

				let selected_rel = source.confirm_pass_directions(available_rels);

				selected_rel
					.into_iter()
					.map(Direction::rel)
					.map(|(rel_x, rel_y)| (rel_x + source_pos.0, rel_y + source_pos.1))
					.collect::<heapless::Vec<_, 4>>()
			};

			for target_pos in selected_target.into_iter() {
				let from =
					Direction::from_rel((source_pos.0 - target_pos.0, source_pos.1 - target_pos.1));

				if self
					.at(target_pos)
					.map(|target| target.can_receive(from))
					.unwrap_or(false)
				{
					moves_queue.multimap_insert(target_pos, source_pos);
				}
			}
		}

		self.moves_queue = moves_queue;
	}

	pub fn grid_at(&self, pos: (i32, i32)) -> Option<&OrIndexed<EBuilding>> {
		self.buildings_grid.at(pos)
	}
	pub fn indexed(&self, index: usize) -> Option<&EBuilding> {
		self.external_buildings.iter().nth(index)
	}
	pub fn insert_indexed(&mut self, building: EBuilding) -> usize {
		// doesn't handle removals at all
		self.external_buildings.push(building);
		self.external_buildings.len() - 1
	}

	pub fn at(&self, pos: (i32, i32)) -> Option<&EBuilding> {
		match self.buildings_grid.at(pos)? {
			OrIndexed::Item(building) => Some(building),
			OrIndexed::Indexed { index: id, .. } => self.external_buildings.iter().nth(*id),
		}
	}
	pub fn at_mut(&mut self, pos: (i32, i32)) -> Option<&mut EBuilding> {
		match self.buildings_grid.at_mut(pos)? {
			OrIndexed::Item(building) => Some(building),
			OrIndexed::Indexed { index: id, .. } => self.external_buildings.iter_mut().nth(*id),
		}
	}

	pub fn iter<'a>(&'a self) -> impl Iterator<Item = ((i32, i32), &'a EBuilding)> + 'a {
		self.buildings_grid
			.iter_coords()
			.map(|pos| (pos, self.at(pos).unwrap()))
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

#[derive(Copy, Clone, Debug)]
/// what to do when we encounter a part of a larger indexed building
pub enum PlaceStrategy {
	Skip,
	SkipIfNotRoot,

	DeleteAll,
}

impl BuildingsMap {
	/// places the building while respecting .is_protected(), and also while dealing
	/// with mutliple-grid entry buildings with PlaceStrategy
	pub fn try_place_explicit(
		&mut self,
		pos: (i32, i32),
		building: OrIndexed<EBuilding>,
		strategy: PlaceStrategy,
	) -> Result<(), OrIndexed<EBuilding>> {
		let protected = self.at(pos).map(Building::is_protected).unwrap_or_default();
		if protected {
			return Err(building);
		}

		let ptr = match self.buildings_grid.at(pos) {
			Some(a) => a,
			None => return Err(building),
		};

		// check if it's a bigger building
		let root = match &*ptr {
			OrIndexed::Indexed { root, .. } => Some(*root),
			OrIndexed::Item(_) => None,
		};
		if let Some(root) = root {
			match strategy {
				PlaceStrategy::Skip => return Err(building),
				PlaceStrategy::SkipIfNotRoot if root != pos => return Err(building),
				_ => {
					let rels = [(0, 0), (1, 0), (0, 1), (1, 1)];
					let rels = rels.into_iter().map(|(rx, ry)| (root.0 + rx, root.1 + ry));
					for place_pos in rels {
						let part_ptr = self.buildings_grid.at_mut(place_pos);
						if let Some(part_ptr) = part_ptr {
							*part_ptr = OrIndexed::Item(EBuilding::nothing())
						}
					}
				}
			}
		}

		let ptr = match self.buildings_grid.at_mut(pos) {
			Some(a) => a,
			None => return Err(building),
		};

		*ptr = building;
		Ok(())
	}

	pub fn try_place(
		&mut self,
		pos: (i32, i32),
		building: OrIndexed<EBuilding>,
	) -> Result<(), OrIndexed<EBuilding>> {
		self.try_place_explicit(pos, building, PlaceStrategy::DeleteAll)
	}
}
