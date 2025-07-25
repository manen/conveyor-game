use crate::{
	game::Game,
	world::buildings::{EBuilding, Nothing, SmallExtractor},
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
	/// temporary tool switch
	pub fn cycle(&mut self) {
		match self {
			Tool::PlaceBuilding(EBuilding::Nothing(_)) => {
				*self = Tool::PlaceBuilding(EBuilding::small_extractor())
			}
			Tool::PlaceBuilding(EBuilding::SmallExtractor(_)) => {
				*self = Tool::PlaceBuilding(EBuilding::debug_consumer())
			}
			Tool::PlaceBuilding(EBuilding::DebugConsumer(_)) => {
				*self = Tool::PlaceBuilding(EBuilding::nothing())
			}
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
