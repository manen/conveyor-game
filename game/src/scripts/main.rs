use std::{fmt::Debug, ops::Deref, path::PathBuf, sync::Arc};

use anyhow::Context;
use asset_provider::Assets;
use stage_manager::StageChange;
use stage_manager_remote::RemoteStage;
use sui::{DynamicLayable, Layable, LayableExt, core::ReturnEvent};
use tokio::sync::Mutex;

use crate::{
	assets::GameAssets,
	comp::{handle_err, handle_result_dyn},
	game::{Game, GameData, GameDataSave},
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
		let tex = tex.expect("fuck");

		let mut game = Game::new(tex);
		game.enable_save_handler(save_handler());

		sui::custom_only_debug(game)
	})
}

fn save_handler() -> impl FnMut(GameData) + Send + 'static {
	let saving: Option<tokio::task::JoinHandle<()>> = None;
	let mut saving = Arc::new(Mutex::new(saving));

	let save_handler = async move |game_data| {
		use rfd::AsyncFileDialog;

		// the actual serialization takes place on another thread while the player's choosing
		// the save location
		let (tx, rx) = tokio::sync::oneshot::channel();
		tokio::task::spawn(async move {
			let f = || {
				let save = GameDataSave::new(&game_data)?;
				let save = bincode::serde::encode_to_vec(&save, bincode::config::standard())?;
				anyhow::Ok(save)
			};
			let save = f();
			let res = tx.send(save);
			match res {
				Ok(_) => {}
				Err(_) => {
					eprintln!(
						"failed to push serialized save into oneshot channel (something's broken)"
					);
				}
			}
		});

		let files = AsyncFileDialog::new()
			.add_filter("save file", &["cgs"])
			.set_directory(std::env::current_dir()?)
			.set_title("saving")
			.set_file_name("new-game.cgs")
			.save_file()
			.await;
		let files = files.with_context(|| format!("file dialog didn't return anthing"))?;
		let path = files.path();
		let path = PathBuf::from(path);

		let save = rx
			.await
			.map(|serialization_err| {
				serialization_err.with_context(|| "failed to serialize GameData")
			})
			.with_context(|| {
				format!("failed to receive from oneshot channel (something's broken)")
			});
		let save = save??;

		tokio::fs::write(&path, &save).await?;
		anyhow::Ok(path)
	};

	let executor = move |game_data: GameData| {
		let mut guard = match saving.try_lock() {
			Ok(a) => a,
			Err(_) => return,
		};
		match guard.deref() {
			Some(handle) => {
				if !handle.is_finished() {
					return;
				}
			}
			None => {}
		};

		let future = save_handler(game_data);

		let handle = tokio::task::spawn(async move {
			match future.await {
				Ok(save_path) => {
					println!("saved to {}", save_path.display())
				}
				Err(err) => {
					eprintln!("failed to save: {err:?}")
				}
			}
		});
		*guard = Some(handle);
	};
	executor
}
