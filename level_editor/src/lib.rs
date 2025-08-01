use std::fmt::Debug;

use game::{
	assets::GameAssets,
	textures::{self, Textures},
};

pub mod level_editor;
use level_editor::LevelEditor;
use sui::Layable;

pub mod tools;

#[tokio::main]
pub async fn start_with_rt() {
	start();
}

pub fn start() {
	let (mut rl, thread) = sui_runner::rl();
	let assets = GameAssets::default();

	let textures = {
		let d = rl.begin_drawing(&thread);
		let mut d = sui::Handle::new_unfocused(d);

		textures::Textures::new(&assets, &mut d, &thread).expect("failed to load textures")
	};
	let game = creation_screen(textures);

	let mut ctx = sui_runner::Context::new(game, rl, thread);
	ctx.start();
}

fn creation_screen(textures: Textures) -> impl Layable + Debug + Clone {
	sui::text("hello we're creating", 32)
}
