use crate::{
	game::Game,
	world::buildings::{EBuilding, Nothing},
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
	pub fn cycle(&mut self) {}

	pub fn r#use(&self, game: &mut Game, pos: (i32, i32)) {
		match self {
			Self::PlaceBuilding(building) => {
				if let Some(r) = game.buildings.at_mut(pos) {
					*r = building.clone()
				}
			}
		}
	}
}
