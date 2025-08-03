use std::fmt::Debug;

use anyhow::Context;
use asset_provider::Assets;
use stage_manager::StageChange;
use stage_manager_loaders::Loader;
use stage_manager_remote::RemoteStageChange;
use sui::{DynamicLayable, Layable, LayableExt};
use tokio::sync::mpsc::{Receiver, Sender};

use crate::{
	assets::GameAssets,
	comp::{handle_err, handle_err_async_dyn, handle_result, handle_result_dyn},
	game::Game,
	levels::{GameState, Level, Levels},
	textures,
	world::maps::BuildingsMap,
};

pub async fn main() -> DynamicLayable<'static> {
	let game_state = GameState::load().await;

	if !game_state.tutorial_completed {
		tutorial().await
	} else {
		DynamicLayable::new_only_debug(main_menu())
	}
}

pub async fn tutorial() -> DynamicLayable<'static> {
	let loader = textures::load_as_layable(GameAssets::default(), |textures| {
		let f = move || {
			let textures = textures?;

			let loader = Loader::new(
				sui::text("loading tutorial...", 16).centered(),
				async move { assemble_tutorial(textures).await },
				|level| StageChange::Simple(handle_result(level)),
			);
			anyhow::Ok(DynamicLayable::new_only_debug(loader))
		};

		handle_result(f())
	});
	DynamicLayable::new_only_debug(loader)
}

pub async fn assemble_tutorial(textures: textures::Textures) -> anyhow::Result<Game> {
	let assets = GameAssets::default();
	let levels = Levels::load(&assets).await?;

	let level = Level::load_from_assets(&assets, &levels.campaign.tutorial).await?;
	let tilemap = level.into_tilemap()?;
	let buildings = BuildingsMap::new(tilemap.width(), tilemap.height());

	let mut game = Game::from_maps(textures, tilemap, buildings);
	game.enable_tips(test_controller);

	Ok(game)
}
pub async fn test_controller(
	tx: Sender<stage_manager_remote::RemoteStageChange>,
	rx: Receiver<()>,
) {
	let mut i = 0;

	loop {
		tx.send(RemoteStageChange::simple(sui::text(
			format!("hi this text is coming from another dimension ({i})"),
			16,
		)))
		.await
		.expect("pocs"); // TODO

		tokio::time::sleep(std::time::Duration::from_millis(1000)).await;
		i += 1;
	}
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
