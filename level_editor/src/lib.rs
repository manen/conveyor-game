use std::{
	error::Error,
	fmt::{Debug, Display},
	sync::Arc,
};

use game::{
	assets::GameAssets,
	textures::{self, Textures},
};

pub mod level_editor;
use level_editor::LevelEditor;
use sui::{Compatible, Layable, LayableExt, core::Store, form::typable::TypableData};

/// safety: only accessed from the main thread
static mut TEXTURES: Option<Textures> = None;

pub mod tools;

#[tokio::main]
pub async fn start_with_rt() {
	start();
}

pub fn start() {
	let (mut rl, thread) = sui_runner::rl();
	let assets = GameAssets::default();

	{
		let d = rl.begin_drawing(&thread);
		let mut d = sui::Handle::new_unfocused(d);

		let textures =
			textures::Textures::new(&assets, &mut d, &thread).expect("failed to load textures");
		unsafe {
			TEXTURES = Some(textures);
		}
	};

	let game = creation_screen();
	let game = stage_manager::Stage::new(game);

	let mut ctx = sui_runner::Context::new(game, rl, thread);
	ctx.start();
}

fn creation_screen() -> impl Layable + Debug + Clone {
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
		sui::custom(sui::text("create level", 32).clickable(move |_| {
			let f = || {
				let width = width_store.with_borrow(|a| a.text.parse())?;
				let height = height_store.with_borrow(|a| a.text.parse())?;

				#[allow(static_mut_refs)]
				let textures = unsafe { TEXTURES.take().expect("textures has already been taken") };

				anyhow::Ok(stage_manager::StageChange::from_dyn(
					sui::DynamicLayable::new_only_debug(LevelEditor::new(width, height, textures)?),
				))
			};

			match f() {
				Ok(a) => a,
				Err(err) => stage_manager::StageChange::new(err_page(err)),
			}
		})),
	])
	.centered()
}

fn err_page<E: Debug + Display>(err: E) -> impl Layable + Debug + Clone {
	let display = format!("{err}");
	let debug = format!("{err:?}");

	let f = move |_| stage_manager::StageChange::new(creation_screen());

	sui::div([
		sui::custom(sui::text(display, 32).centered()),
		sui::custom(sui::text(debug, 24)),
		sui::DynamicLayable::new_only_debug(sui::text("return to main menu", 24).clickable(f)),
	])
}
