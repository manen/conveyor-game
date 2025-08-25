use std::{borrow::Cow, fmt::Debug};

use crate::{
	GameData,
	buildings::{Building, EBuilding, Nothing},
	maps::OrIndexed,
	render::TILE_RENDER_SIZE,
};
use sui::{Details, Layable, LayableExt};
use textures::{TextureID, Textures};
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
				let index = game.buildings.insert_indexed(building.clone());
				let indexed = OrIndexed::Indexed { index, root: pos };

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
				mklogger::eprintln!("failed to place {err:?}")
			}
		}
	}
	// pub fn held(&self, game: &mut GameData, pos: (i32, i32)) {}
	// pub fn release(&mut self, game: &mut GameData, pos: (i32, i32)) {}
}

impl Tool {
	pub fn render_preview<'a>(
		&'a self,
		textures: &'a Textures,
		world_size: (usize, usize),
		hovering_over: (i32, i32),
	) -> impl Layable + Debug + Clone + 'a {
		#[derive(Clone, Debug)]
		pub struct RenderPreview<'a> {
			textures: &'a Textures,
			world_size: (usize, usize),
			hovering_over: (i32, i32),

			tool: &'a Tool,
		}
		impl<'a> Layable for RenderPreview<'a> {
			fn size(&self) -> (i32, i32) {
				(
					self.world_size.0 as i32 * TILE_RENDER_SIZE,
					self.world_size.1 as i32 * TILE_RENDER_SIZE,
				)
			}
			fn render(&self, d: &mut sui::Handle, det: sui::Details, scale: f32) {
				let render_size = TILE_RENDER_SIZE as f32 * scale;
				let render_size_i32 = render_size as i32;

				let det_for_coord = |(x, y)| {
					let draw_x = det.x + (x as f32 * render_size) as i32;
					let draw_y = det.y + (y as f32 * render_size) as i32;

					let (draw_x, draw_y) = (draw_x - 1, draw_y - 1);
					let render_size_i32 = render_size_i32 + 1;

					let l_det = Details {
						x: draw_x,
						y: draw_y,
						aw: render_size_i32,
						ah: render_size_i32,
					};
					l_det
				};
				let l_det = det_for_coord(self.hovering_over);

				match self.tool {
					Tool::PlaceBuilding(building) => {
						building.render(self.textures).render(d, l_det, scale);
					}
					Tool::Place2x2(building) => {
						building
							.render(self.textures)
							.render(d, l_det.mul_size(2.0), scale);
					}
				}
			}
		}

		RenderPreview {
			textures,
			tool: self,

			world_size,
			hovering_over,
		}
	}
}
