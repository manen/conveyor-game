use std::fmt::Debug;

use stage_manager::StageChange;
use stage_manager_loaders::Loader;
use stage_manager_remote::StageSyncWrap;
use sui::{DynamicLayable, Layable, LayableExt};
// use tokio::sync::mpsc::{Receiver, Sender};

mod controller;
use controller::controller;
use tokio::sync::mpsc;

use crate::{
	assets::GameAssets,
	comp::handle_result,
	game::{Game, GameRunner, Goal, goal::ResourceCounter},
	levels::{Level, Levels},
	scripts::tutorial,
	textures,
	world::{
		buildings::{ChannelConsumer, EBuilding},
		maps::BuildingsMap,
	},
};

pub fn tutorial() -> DynamicLayable<'static> {
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

pub async fn assemble_tutorial(
	textures: textures::Textures,
) -> anyhow::Result<impl Layable + Debug> {
	let assets = GameAssets::default();
	let levels = Levels::load(&assets).await?;

	let level = Level::load_from_assets(&assets, &levels.campaign.tutorial).await?;
	let tilemap = level.into_tilemap()?;
	let tilemap_size = tilemap.size();

	let mut buildings = BuildingsMap::new(tilemap_size.0, tilemap_size.1);

	let (master_tx, master_rx) = mpsc::channel(5);

	let (mut consumer, resources_rx) = ChannelConsumer::new();
	consumer.protected = true;
	let consumer = EBuilding::ChannelConsumer(consumer);
	place_at_center(&mut buildings, consumer);

	let game = Game::from_maps(textures.clone(), tilemap, buildings);
	let (mut game, game_tx) = GameRunner::new(game);

	let tool_use_rx = game.subscribe_to_tool_use();
	game.enable_tips_spawn(|tx, rx| {
		let channels = controller::Channels {
			textures,
			goal: ResourceCounter::new(Goal::new([]), resources_rx),
			master_tx,

			stage_size: tilemap_size,
			stage_tx: tx,
			stage_rx: rx,
			tool_use_rx,
			game_tx,
		};

		controller::controller(channels)
	});

	let game = StageSyncWrap::assemble(game, master_rx);
	Ok(game)
}

pub(self) fn place_at_center(buildings: &mut BuildingsMap, building: EBuilding) {
	let (w, h) = buildings.size();

	let (center_x, center_y) = (w as f32 / 2.0, h as f32 / 2.0);
	let (center_x, center_y) = (center_x - 0.5, center_y - 0.5);

	let (center_x_floor, center_x_ceil) = (center_x.floor(), center_x.ceil());
	let (center_y_floor, center_y_ceil) = (center_y.floor(), center_y.ceil());
	let (center_x, center_y) = (center_x as i32, center_y as i32);

	// // println!("{center_x_floor}-{center_x_ceil}, {center_y_floor}-{center_y_ceil}");
	// place_buildings(
	// 	buildings,
	// 	[
	// 		(center_x_floor as i32, center_y_floor as i32),
	// 		(center_x_floor as i32, center_y_ceil as i32),
	// 		(center_x_ceil as i32, center_y_floor as i32),
	// 		(center_x_ceil as i32, center_y_ceil as i32),
	// 	],
	// 	building,
	// );

	let eqs = (
		center_x_floor == center_x_ceil,
		center_y_floor == center_y_ceil,
	);
	// println!("eqs: {eqs:?}, size: ({w}, {h})");
	match eqs {
		(true, true) => {
			// there's a single block in the center
			let coords = [(center_x, center_y)];
			place_buildings(buildings, coords, building);
		}
		(true, false) => {
			let coords = [
				(center_x, center_y_floor as i32),
				(center_x, center_y_ceil as i32),
			];
			place_buildings(buildings, coords, building);
		}
		(false, true) => {
			let coords = [
				(center_x_floor as i32, center_y),
				(center_x_ceil as i32, center_y),
			];
			place_buildings(buildings, coords, building);
		}
		(false, false) => {
			let coords = [
				(center_x_floor as i32, center_y_floor as i32),
				(center_x_floor as i32, center_y_ceil as i32),
				(center_x_ceil as i32, center_y_floor as i32),
				(center_x_ceil as i32, center_y_ceil as i32),
			];
			place_buildings(buildings, coords, building);
		}
	};
}
fn place_buildings(
	buildings: &mut BuildingsMap,
	iter: impl IntoIterator<Item = (i32, i32)>,
	building: EBuilding,
) {
	for coords in iter {
		*buildings
			.at_mut(coords)
			.expect("out of range coordinates passed to place_buildings") = building.clone();
	}
}
