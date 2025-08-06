use stage_manager::StageChange;
use stage_manager_loaders::Loader;
use sui::{DynamicLayable, LayableExt};
// use tokio::sync::mpsc::{Receiver, Sender};

mod tips;
use tips::controller;

use crate::{
	assets::GameAssets,
	comp::handle_result,
	game::Game,
	levels::{Level, Levels},
	textures,
	world::{
		buildings::{ChannelConsumer, EBuilding},
		maps::BuildingsMap,
	},
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

	let mut buildings = BuildingsMap::new(tilemap.width(), tilemap.height());

	let (mut consumer, resource_rx) = ChannelConsumer::new();
	consumer.protected = true;
	let consumer = EBuilding::ChannelConsumer(consumer);
	place_at_center(&mut buildings, consumer);

	let mut game = Game::from_maps(textures, tilemap, buildings);

	let tool_use_rx = game.subscribe_to_tool_use();
	game.enable_tips(|tx, rx| controller(tx, rx, tool_use_rx));

	Ok(game)
}

fn place_at_center(buildings: &mut BuildingsMap, building: EBuilding) {
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
