use std::borrow::Cow;

use crate::{
	game::Game,
	textures::TextureID,
	world::buildings::{Building, EBuilding, Nothing},
};

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

	pub fn r#use(&self, game: &mut Game, pos: (i32, i32)) {
		match self {
			Self::PlaceBuilding(EBuilding::Nothing(_)) => {
				if let Some(existing) = game.buildings.at_mut(pos) {
					if !existing.is_protected() {
						*existing = EBuilding::nothing();
					}
				}
			}
			Self::PlaceBuilding(building) => {
				if let Some(r) = game.buildings.at_mut(pos) {
					*r = building.clone()
				} else {
					eprintln!("placing building on invalid position: {pos:?}")
				}
			}
		}
	}
}
