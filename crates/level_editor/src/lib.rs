use std::{
	borrow::Cow,
	fmt::{Debug, Display},
	sync::Arc,
};

use anyhow::Context;
use game::{
	assets::GameAssets,
	textures,
	world::maps::{Tilemap, TilemapExt},
};
use utils::SilentUnwrap;

pub mod level_editor;
use level_editor::LevelEditor;
use stage_manager::StageChange;
use stage_manager_loaders::Loader;
use sui::{Compatible, Layable, LayableExt, core::Store, form::typable::TypableData};

pub mod tools;

#[tokio::main]
pub async fn start_with_rt() {
	start().await;
}

pub async fn start() {
	let (rl, thread) = sui_runner::rl();

	game_worldgen::init_worldgen(&GameAssets::default())
		.await
		.silent_unwrap();

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
		sui::custom_only_debug(
			sui::div([
				sui::custom_only_debug(generate_button(
					"new empty",
					width_store.clone(),
					height_store.clone(),
					|width, height| Ok::<_, String>(Tilemap::stone(width, height)),
				)),
				sui::custom_only_debug(generate_button(
					"generate",
					width_store,
					height_store,
					game_worldgen::gen_world,
				)),
			])
			.margin(4),
		),
		sui::custom(sui::text("or", 32).center_y().margin(32)),
		sui::custom_only_debug(sui::text("load from file", 32).clickable(move |_| open_screen())),
	])
	.centered()
}

fn generate_button<
	E: Debug + Display + 'static,
	N: Into<Cow<'static, str>>,
	F: Fn(usize, usize) -> Result<Tilemap, E> + 'static,
>(
	name: N,
	width: Store<TypableData>,
	height: Store<TypableData>,
	gen_tilemap: F,
) -> impl Layable + Debug + 'static {
	let gen_tilemap = Arc::new(gen_tilemap);

	sui::text(name, 32)
		.clickable(move |_| {
			let width = width.clone();
			let height = height.clone();
			let gen_tilemap = gen_tilemap.clone();

			let f = move || {
				let width = width.with_borrow(|a| a.text.parse())?;
				let height = height.with_borrow(|a| a.text.parse())?;

				anyhow::Ok(textures::load_as_scene(
					GameAssets::default(),
					move |tex| match tex {
						Ok(textures) => {
							let tilemap = gen_tilemap(width, height);
							let tilemap = match tilemap {
								Ok(a) => a,
								Err(err) => return sui::custom_only_debug(err_page(err)),
							};
							sui::custom_only_debug(LevelEditor::from_tilemap(tilemap, textures))
						}
						Err(err) => sui::custom_only_debug(err_page(err)),
					},
				))
			};

			match f() {
				Ok(a) => a,
				Err(err) => StageChange::simple_only_debug(err_page(err)),
			}
		})
		.margin(3)
}

fn open_screen() -> StageChange<'static> {
	let loading = Loader::new_overlay(
		sui::text("select save on file picker", 32).centered(),
		async {
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

			let file = tokio::fs::OpenOptions::new()
				.read(true)
				.open(file.path())
				.await?;
			let mut file = file.into_std().await;

			let tilemap: Tilemap =
				bincode::serde::decode_from_std_read(&mut file, bincode::config::standard())?;

			anyhow::Ok(tilemap)
		},
		|p| match p {
			Ok(tilemap) => textures::load_as_scene(GameAssets::default(), move |tex| match tex {
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
	comp_extra::err_page_customizable(
		err,
		Some(stage_manager::StageChange::simple_only_debug(
			creation_screen(),
		)),
	)
}
