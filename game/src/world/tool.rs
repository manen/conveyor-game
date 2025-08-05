use std::borrow::Cow;

use crate::{
	game::Game,
	utils::Direction,
	world::buildings::{Building, Conveyor, EBuilding, Nothing},
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

	pub fn r#use(&self, game: &mut Game, pos: (i32, i32)) {
		match self {
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
