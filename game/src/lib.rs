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

	let (rl, thread) = sui_runner::rl();

	let stage = stage_manager::Stage::new_only_debug(comp::main_menu());
	let mut ctx = sui_runner::Context::new(stage, rl, thread);

	ctx.start();
}
