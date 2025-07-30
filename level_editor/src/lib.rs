use game::assets::GameAssets;

pub mod level_editor;
use level_editor::LevelEditor;

pub mod tools;

#[tokio::main]
pub async fn start_with_rt() {
	start();
}

pub fn start() {
	let (mut rl, thread) = sui_runner::rl();
	let assets = GameAssets::default();

	let game = {
		let d = rl.begin_drawing(&thread);
		let mut d = sui::Handle::new_unfocused(d);

		LevelEditor::new(&assets, &mut d, &thread).unwrap()
	};

	let mut ctx = sui_runner::Context::new(game, rl, thread);
	ctx.start();
}
