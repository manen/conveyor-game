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
	start().await;
}

pub async fn start() {
	println!("Hello, world!");

	let (rl, thread) = sui_runner::rl();

	let stage = stage_manager::Stage::from_dyn_layable(comp::main().await);

	let mut ctx = sui_runner::Context::new(stage, rl, thread);

	ctx.start();
}
