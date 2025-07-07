pub mod tile;

fn main() {
	println!("Hello, world!");

	let game = game::game();
	sui_runner::ctx(game).start();
}

mod game {
	use stage_manager::{Stage, StageChange};
	use std::fmt::Debug;
	use sui::{Layable, LayableExt, Text};

	pub fn game() -> impl Layable {
		Stage::new(page1())
	}

	pub fn page1() -> impl Layable + Debug + Clone {
		sui::div([
			sui::custom(Text::new("hello bello", 24)),
			sui::custom(
				Text::new("click here to go to the moon", 18)
					.margin(4)
					.clickable(|_| StageChange::new(page2())),
			),
		])
	}

	pub fn page2() -> impl Layable + Debug + Clone {
		sui::div([
			sui::custom(Text::new(
				"oh well we didn't quite make it to the moon but whatever",
				18,
			)),
			sui::custom(
				Text::new("click here to go back", 32)
					.margin(4)
					.clickable(|_| StageChange::new(page1())),
			),
		])
	}
}
