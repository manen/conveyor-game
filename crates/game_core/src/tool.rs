use std::borrow::Cow;

use crate::{
	GameData,
	buildings::{Building, EBuilding, Nothing},
	maps::OrIndexed,
};
use textures::TextureID;
use utils::Direction;

pub fn tools() -> impl Iterator<Item = Tool> {
	use std::iter;

	iter::once(Tool::PlaceBuilding(EBuilding::nothing()))
		.chain(Direction::all().map(|dir| Tool::PlaceBuilding(EBuilding::conveyor(dir))))
		.chain([
			Tool::Place2x2(EBuilding::small_extractor()),
			Tool::PlaceBuilding(EBuilding::debug_consumer()),
			Tool::PlaceBuilding(EBuilding::junction()),
			Tool::PlaceBuilding(EBuilding::router()),
			Tool::PlaceBuilding(EBuilding::smelter()),
		])
}

#[derive(Clone, Debug)]
pub enum Tool {
	PlaceBuilding(EBuilding),
	/// places 4 buildings and hooks them up to the same building impl
	Place2x2(EBuilding),
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

			Tool::Place2x2(EBuilding::Nothing(_)) => "remove buildings".into(),
			Tool::Place2x2(building) => format!("place {}", building.name()).into(),
		}
	}
	pub fn texture_id(&self) -> TextureID {
		match self {
			Tool::PlaceBuilding(building) => building.texture_id(),
			Tool::Place2x2(building) => building.texture_id(),
		}
	}

	pub fn r#use(&self, game: &mut GameData, pos: (i32, i32)) {
		let mut f = || match self {
			Self::PlaceBuilding(building) => game
				.buildings
				.try_place(pos, OrIndexed::Item(building.clone())),

			Self::Place2x2(building) => {
				let indexed = game.buildings.insert_indexed(building.clone());
				let indexed = OrIndexed::Indexed(indexed);

				let rels = [(0, 0), (1, 0), (0, 1), (1, 1)];
				let rels = rels.into_iter().map(|(rx, ry)| (pos.0 + rx, pos.1 + ry));

				for place_pos in rels {
					game.buildings.try_place(place_pos, indexed.clone())?;
				}
				Ok(())
			}
		};

		match f() {
			Ok(_) => {}
			Err(err) => {
				eprintln!("failed to place {err:?}")
			}
		}
	}
	// pub fn held(&self, game: &mut GameData, pos: (i32, i32)) {}
	// pub fn release(&mut self, game: &mut GameData, pos: (i32, i32)) {}
}
