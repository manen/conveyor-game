use std::fmt::{Debug, Display};

use anyhow::Context;
use game::{assets::GameAssets, textures::loader::load_as_scene as load_textures};

pub mod level_editor;
use level_editor::LevelEditor;
use stage_manager::StageChange;
use stage_manager_tokio::Loader;
use sui::{Compatible, Layable, LayableExt, core::Store, form::typable::TypableData};

pub mod tools;

#[tokio::main]
pub async fn start_with_rt() {
	start();
}

pub fn start() {
	let (rl, thread) = sui_runner::rl();

	let game = creation_screen();
	let game = stage_manager::Stage::new_only_debug(game);

	let mut ctx = sui_runner::Context::new(game, rl, thread);
	ctx.start();
}

fn creation_screen() -> impl Layable + Debug {
	let width_store = Store::new(TypableData::with_default(format!(
		"{}",
		game::world::maps::SIZE
	)));
	let height_store = Store::new(TypableData::with_default(format!(
		"{}",
		game::world::maps::SIZE
	)));

	sui::div([
		sui::custom(sui::Text::new("hello we're creating", 32)),
		sui::custom(sui::div_h([
			sui::text("width: ", 24),
			sui::custom(sui::form::textbox(width_store.clone(), 24)).into_comp(),
		])),
		sui::custom(sui::div_h([
			sui::text("height: ", 24),
			sui::custom(sui::form::textbox(height_store.clone(), 24)).into_comp(),
		])),
		//
		sui::custom_only_debug(sui::text("create level", 32).clickable(move |_| {
			let f = || {
				let width = width_store.with_borrow(|a| a.text.parse())?;
				let height = height_store.with_borrow(|a| a.text.parse())?;

				anyhow::Ok(load_textures(GameAssets::default(), move |tex| match tex {
					Ok(tex) => {
						sui::DynamicLayable::new_only_debug(LevelEditor::new(width, height, tex))
					}
					Err(err) => sui::custom_only_debug(err_page(err)),
				}))
			};

			match f() {
				Ok(a) => a,
				Err(err) => StageChange::simple_only_debug(err_page(err)),
			}
		})),
		sui::custom(sui::text("or", 32).centered().fix_wh(300, 200)),
		sui::custom_only_debug(sui::text("load from file", 32).clickable(move |_| open_screen())),
	])
	.centered()
}

fn open_screen() -> StageChange<'static> {
	let loading = Loader::new_overlay(
		sui::text("loading save...", 32).centered(),
		async {
			use game::levels::Level;
			use rfd::AsyncFileDialog;

			let picker = AsyncFileDialog::new()
				.add_filter("level file", &["cglf"])
				.set_directory(std::env::current_dir()?)
				.set_title("select level to load")
				.pick_file()
				.await;
			let file = picker
				.with_context(|| format!("AsyncFileDialog didn't return a file handle"))
				.with_context(|| format!("failed to open file"))?;
			let save = Level::load(std::path::PathBuf::from(file.path())).await?;

			save.into_tilemap()
		},
		|p| match p {
			Ok(tilemap) => load_textures(GameAssets::default(), move |tex| match tex {
				Ok(tex) => {
					let level_editor = LevelEditor::from_tilemap(tilemap.clone(), tex);
					sui::DynamicLayable::new_only_debug(level_editor)
				}
				Err(err) => sui::custom_only_debug(err_page(err)),
			}),
			Err(err) => StageChange::Simple(sui::custom_only_debug(err_page(err))),
		},
	);

	loading
}

fn err_page<E: Debug + Display>(err: E) -> impl Layable + Debug {
	game::comp::err_page(
		err,
		Some(stage_manager::StageChange::simple_only_debug(
			creation_screen(),
		)),
	)
}
