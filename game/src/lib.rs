use sui::DynamicLayable;

use crate::{assets::GameAssets, game::Game};

pub mod assets;
pub mod comp;
pub mod game;
pub mod levels;
pub mod textures;
pub mod utils;
pub mod world;

#[tokio::main]
pub async fn start_with_rt() {
	start();
}

pub fn start() {
	println!("Hello, world!");

	let (mut rl, thread) = sui_runner::rl();

	let assets = GameAssets::default();

	let game = {
		let d = rl.begin_drawing(&thread);
		let fh = sui::core::Store::new(sui::form::UniqueId::null());
		let mut d = sui::Handle::new(d, &fh);

		Game::new(&assets, &mut d, &thread).unwrap()
	};

	let stage = stage_manager::Stage::from_dyn_layable(DynamicLayable::new_only_debug(game));
	let mut ctx = sui_runner::Context::new(stage, rl, thread);

	ctx.start();
}
