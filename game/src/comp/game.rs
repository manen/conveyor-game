use std::fmt::Debug;

use stage_manager::StageChange;
use sui::{Layable, LayableExt};

use crate::{assets::GameAssets, game::Game, textures};

pub fn main_menu() -> impl Layable + Debug {
	let title = sui::text("conveyor-game", 32);
	let load_game = sui::text("load into game", 16).clickable(|_| game());

	let container = sui::div([sui::custom(title), sui::custom_only_debug(load_game)]);

	container.centered()
}

pub fn game() -> StageChange<'static> {
	textures::load_as_scene(GameAssets::default(), |tex| {
		let tex = tex.expect("fuck"); // TODO
		sui::custom_only_debug(Game::new(tex))
	})
}
