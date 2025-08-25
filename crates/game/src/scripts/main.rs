use std::{fmt::Debug, io::Seek, ops::Deref, path::PathBuf, sync::Arc};

use anyhow::Context;
use asset_provider::Assets;
use stage_manager::StageChange;
use sui::{DynamicLayable, Layable, LayableExt, core::ReturnEvent};
use tokio::sync::Mutex;

use crate::{
	assets::GameAssets,
	comp::{err_page, handle_err, handle_result_dyn},
	game::{Game, GameData, GameDataSave},
	levels::GameState,
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

pub async fn main_menu() -> impl Layable + Debug {
	let game_state = GameState::load().await;
	let only_allow_tutorial = !game_state.tutorial_completed;

	let title = sui::Text::new("conveyor-game", 32);
	let title = title.margin(32);
	let title = title.center_x();

	let start_tutorial = comp_extra::button_explicit("start tutorial", false, || {
		let tutorial = tutorial::tutorial();
		ReturnEvent::new(StageChange::Simple(tutorial))
	});
	let start_freeplay = comp_extra::button_explicit("free play", only_allow_tutorial, || {
		ReturnEvent::new(free_play(None))
	});
	let load_freeplay =
		comp_extra::button_explicit("load save file", only_allow_tutorial, freeplay_loader);

	let buttons = sui::div([
		sui::custom_only_debug(start_tutorial),
		sui::custom_only_debug(start_freeplay),
		sui::custom_only_debug(load_freeplay),
	]);
	let buttons = buttons.restrict_to_size().center_x();
	let buttons = buttons.center_y();

	let page = buttons.overlay(title);
	page
}

pub fn free_play(game_data: Option<GameData>) -> StageChange<'static> {
	let (tx, mut rx) = tokio::sync::oneshot::channel();
	let _ = tx.send(game_data);
	textures::load_as_scene(GameAssets::default(), move |tex| {
		let tex = tex.expect("fuck");

		let recv_msg = "failed to receive Option<GameData> from oneshot channel";
		let game = rx.try_recv().with_context(|| recv_msg);
		let game = match game {
			Ok(a) => a,
			Err(err) => {
				mklogger::eprintln!("{}", recv_msg);
				let err_page = err_page(err);
				return sui::custom_only_debug(err_page);
			}
		};
		let mut game = match game {
			Some(data) => Game::new_multithread(tex, data),
			None => match Game::new_multithread_worldgen(tex) {
				Ok(a) => a,
				Err(err) => {
					mklogger::eprintln!("{err}");
					let err_page = err_page(err);
					return sui::custom_only_debug(err_page);
				}
			},
		};
		game.enable_save_handler(save_handler());

		sui::custom_only_debug(game)
	})
}

fn save_handler() -> impl FnMut(GameData) + Send + 'static {
	let saving: Option<tokio::task::JoinHandle<()>> = None;
	let saving = Arc::new(Mutex::new(saving));

	let save_handler = async move |game_data| {
		use rfd::AsyncFileDialog;

		// the actual serialization takes place on another thread while the player's choosing
		// the save location
		let (tx, rx) = tokio::sync::oneshot::channel();
		tokio::task::spawn(async move {
			let f = || {
				let save = GameDataSave::new(&game_data)?;
				let mut buf = Vec::new();
				save.save(&mut buf)?;
				anyhow::Ok(buf)
			};
			let save = f();
			let res = tx.send(save);
			match res {
				Ok(_) => {}
				Err(_) => {
					mklogger::eprintln!(
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
					let display = save_path.display();
					mklogger::println!("saved to {display}")
				}
				Err(err) => {
					mklogger::eprintln!("failed to save: {err:?}")
				}
			}
		});
		*guard = Some(handle);
	};
	executor
}

fn freeplay_loader() -> ReturnEvent {
	let future = async {
		use rfd::AsyncFileDialog;

		let files = AsyncFileDialog::new()
			.add_filter("save file", &["cgs"])
			.set_directory(std::env::current_dir()?)
			.set_title("saving")
			.set_file_name("new-game.cgs")
			.pick_file()
			.await;
		let files = files.with_context(|| format!("file dialog didn't return anthing"))?;
		let path = files.path();
		let path = PathBuf::from(path);

		let file = tokio::fs::OpenOptions::new()
			.read(true)
			.open(&path)
			.await
			.with_context(|| format!("while opening save file at {}", path.display()))?;
		let mut file = file.into_std().await;

		let decoded = GameDataSave::load_as_either(&mut file)?;
		let game_data = decoded.take()?;

		anyhow::Ok(game_data)
	};
	let post_process = move |res| match res {
		Ok(game_data) => free_play(Some(game_data)),
		Err(err) => {
			let err_page = err_page(err);
			StageChange::simple_only_debug(err_page)
		}
	};

	let loader = stage_manager_loaders::Loader::new_invisible(future, post_process);
	ReturnEvent::new(loader)
}
