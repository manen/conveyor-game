use std::borrow::Cow;

use crate::{
	GameData,
	buildings::{Building, EBuilding, Nothing},
};
use textures::TextureID;
use utils::Direction;

pub fn tools() -> impl Iterator<Item = Tool> {
	use std::iter;

	iter::once(Tool::PlaceBuilding(EBuilding::nothing()))
		.chain(Direction::all().map(|dir| Tool::PlaceBuilding(EBuilding::conveyor(dir))))
		.chain([
			Tool::PlaceBuilding(EBuilding::small_extractor()),
			Tool::PlaceBuilding(EBuilding::debug_consumer()),
			Tool::PlaceBuilding(EBuilding::junction()),
			Tool::PlaceBuilding(EBuilding::router()),
			Tool::PlaceBuilding(EBuilding::smelter()),
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

	pub fn r#use(&self, game: &mut GameData, pos: (i32, i32)) {
		match self {
			Self::PlaceBuilding(building) => match building {
				_ => {
					if let Some(existing) = game.buildings.at_mut(pos) {
						if !existing.is_protected() {
							*existing = building.clone()
						}
					} else {
						eprintln!("placing building on invalid position: {pos:?}")
					}
				}
			},
		}
	}
	// pub fn held(&self, game: &mut GameData, pos: (i32, i32)) {}
	// pub fn release(&mut self, game: &mut GameData, pos: (i32, i32)) {}
}
