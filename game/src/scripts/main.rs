use std::fmt::Debug;

use anyhow::Context;
use asset_provider::Assets;
use stage_manager::StageChange;
use sui::{DynamicLayable, Layable, LayableExt, core::ReturnEvent};

use crate::{
	assets::GameAssets,
	comp::{handle_err, handle_result_dyn},
	game::Game,
	levels::{GameState, Level},
	scripts::tutorial,
	textures,
	world::maps::BuildingsMap,
};

pub async fn main() -> DynamicLayable<'static> {
	let main_menu = main_menu().await;

	let main = main_menu;
	let main = sui::custom_only_debug(main);
	main
}

pub async fn level_by_id<A: Assets + Send + Sync + 'static>(
	assets: A,
	id: &str,
) -> DynamicLayable<'static> {
	let f = async move || {
		let level = Level::load_from_assets(&assets, id)
			.await
			.with_context(|| format!("while loading level by id"))?;

		let tilemap = level.into_tilemap()?;
		let buildings = BuildingsMap::new(tilemap.width(), tilemap.height());

		let loader = textures::load_as_layable(assets, move |tex| {
			let f = || {
				let tex = tex?;

				let game = Game::from_maps(tex, tilemap.clone(), buildings.clone());
				anyhow::Ok(game)
			};
			handle_err(f)
		});
		let loader = sui::custom_only_debug(loader);

		anyhow::Ok(loader)
	};

	handle_result_dyn(f().await)
}

pub async fn main_menu() -> impl Layable + Debug {
	let game_state = GameState::load().await;
	let only_allow_tutorial = !game_state.tutorial_completed;

	let title = sui::Text::new("conveyor-game", 32);
	let title = title.margin(32);
	let title = title.center_x();

	let start_tutorial = crate::comp::button_explicit("start tutorial", false, || {
		let tutorial = tutorial::tutorial();
		ReturnEvent::new(StageChange::Simple(tutorial))
	});
	let start_freeplay =
		crate::comp::button_explicit(
			"free play",
			only_allow_tutorial,
			|| ReturnEvent::new(game()),
		);

	let buttons = sui::div([
		sui::custom_only_debug(start_tutorial),
		sui::custom_only_debug(start_freeplay),
	]);
	let buttons = buttons.restrict_to_size().center_x();
	let buttons = buttons.center_y();

	let page = buttons.overlay(title);
	page
}

pub fn game() -> StageChange<'static> {
	textures::load_as_scene(GameAssets::default(), |tex| {
		let tex = tex.expect("fuck"); // TODO
		sui::custom_only_debug(Game::new(tex))
	})
}
