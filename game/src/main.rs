use sui::{Layable, Text};

fn main() {
	println!("Hello, world!");

	let game = game();
	sui_runner::ctx(game).start();
}

fn game() -> impl Layable {
	Text::new("hello", 32)
}
