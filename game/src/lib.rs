use stage_manager::StageChange;
use sui::LayableExt;

use crate::{assets::GameAssets, game::Game};

pub mod assets;
pub mod game;
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

	let components: [sui::Comp<'static>; 6] = core::array::from_fn(|i| {
		sui::text(
			"very very very long text that i'm sure you won't just scroll over and will read to its entireity",
			(i as i32 + 2) * 3,
		)
	});
	let mut game = Some(game);
	let main_menu = sui::div_h([
		sui::custom(sui::div_h(components)),
		sui::custom(
			sui::text("load into game", 32)
				.clickable(move |_| StageChange::new(game.take().unwrap())),
		),
	])
	.scrollable_horiz(Default::default());

	let stage = stage_manager::Stage::new(main_menu);
	let mut ctx = sui_runner::Context::new(stage, rl, thread);

	ctx.start();
}
