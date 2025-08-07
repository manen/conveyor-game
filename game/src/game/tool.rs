use std::borrow::Cow;

use crate::{
	game::Game,
	textures::TextureID,
	utils::Direction,
	world::buildings::{Building, EBuilding, Nothing},
};

pub fn tools() -> impl Iterator<Item = Tool> {
	use std::iter;

	iter::once(Tool::PlaceBuilding(EBuilding::nothing()))
		.chain(Direction::all().map(|dir| Tool::PlaceBuilding(EBuilding::conveyor(dir))))
		.chain([
			Tool::PlaceBuilding(EBuilding::small_extractor()),
			Tool::PlaceBuilding(EBuilding::debug_consumer()),
			Tool::PlaceBuilding(EBuilding::junction()),
		])
}

#[derive(Clone, Debug)]
pub enum Tool {
	PlaceBuilding(EBuilding),
}
impl Default for Tool {
	fn default() -> Self {
		Self::PlaceBuilding(EBuilding::Nothing(Nothing))
	}
}
impl Tool {
	pub fn name(&self) -> Cow<'static, str> {
		match self {
			Tool::PlaceBuilding(EBuilding::Nothing(_)) => "remove buildings".into(),
			Tool::PlaceBuilding(building) => format!("place {}", building.name()).into(),
		}
	}
	pub fn texture_id(&self) -> TextureID {
		match self {
			Tool::PlaceBuilding(building) => building.texture_id(),
		}
	}

	pub fn r#use(&mut self, game: &mut Game, pos: (i32, i32)) {
		match self {
			Self::PlaceBuilding(EBuilding::Nothing(_)) => {
				if let Some(existing) = game.buildings.at_mut(pos) {
					if !existing.is_protected() {
						*existing = EBuilding::nothing();
					}
				}
			}
			Self::PlaceBuilding(building) => match building {
				_ => {
					if let Some(r) = game.buildings.at_mut(pos) {
						*r = building.clone()
					} else {
						eprintln!("placing building on invalid position: {pos:?}")
					}
				}
			},
		}
	}
	pub fn held(&self, game: &mut Game, pos: (i32, i32)) {}
	pub fn release(&mut self, game: &mut Game, pos: (i32, i32)) {}
}
