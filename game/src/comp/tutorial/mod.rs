use stage_manager::StageChange;
use stage_manager_loaders::Loader;
use stage_manager_remote::RemoteStageChange;
use sui::{DynamicLayable, LayableExt};
use tokio::sync::mpsc::{Receiver, Sender};

mod tips;
use tips::controller;

use crate::{
	assets::GameAssets,
	comp::handle_result,
	game::Game,
	levels::{Level, Levels},
	textures,
	world::maps::BuildingsMap,
};

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

	let tool_use_rx = game.subscribe_to_tool_use();
	game.enable_tips(|tx, rx| controller(tx, rx, tool_use_rx));

	Ok(game)
}
