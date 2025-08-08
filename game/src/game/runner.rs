use sui::Layable;
use tokio::sync::mpsc;

use super::Game;

pub struct GameCommand(pub Box<dyn FnOnce(&mut Game)>);

#[derive(Debug)]
/// a wrapper around Game that allows you to call Game functions from another thread
pub struct GameRunner {
	game: Game,
	rx: mpsc::Receiver<GameCommand>,
}
impl GameRunner {
	pub fn new(game: Game) -> (Self, mpsc::Sender<GameCommand>) {
		let (tx, rx) = tokio::sync::mpsc::channel(10);

		(Self { game, rx }, tx)
	}
}
impl Layable for GameRunner {
	fn size(&self) -> (i32, i32) {
		self.game.size()
	}
	fn render(&self, d: &mut sui::Handle, det: sui::Details, scale: f32) {
		self.game.render(d, det, scale);
	}

	fn tick(&mut self) {
		match self.rx.try_recv() {
			Ok(command) => {
				let f = command.0;
				f(&mut self.game)
			}
			Err(_) => {}
		}

		self.game.tick()
	}
	fn pass_events(
		&mut self,
		events: impl Iterator<Item = sui::core::Event>,
		det: sui::Details,
		scale: f32,
		ret_events: &mut Vec<sui::core::ReturnEvent>,
	) {
		self.game.pass_events(events, det, scale, ret_events);
	}
}
