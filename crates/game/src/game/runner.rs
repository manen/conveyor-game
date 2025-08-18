use std::{
	fmt::Debug,
	ops::{Deref, DerefMut},
};

use game_core::GameProvider;
use sui::Layable;
use tokio::sync::{mpsc, oneshot};

use super::Game;

pub struct GameCommand<G: GameProvider>(pub Box<dyn FnOnce(&mut Game<G>) + Send>);
impl<G: GameProvider> GameCommand<G> {
	pub fn new<F: FnOnce(&mut Game<G>) + Send + 'static>(f: F) -> Self {
		Self(Box::new(f))
	}

	pub fn new_return<R: Debug + Send + 'static, F: FnOnce(&mut Game<G>) -> R + Send + 'static>(
		f: F,
	) -> (Self, oneshot::Receiver<R>) {
		let (tx, rx) = oneshot::channel();

		let command = Self::new(move |game| {
			let ret = f(game);
			tx.send(ret)
				.expect("could not send return value into oneshot channel from GameCommand");
		});

		(command, rx)
	}
}

#[derive(Debug)]
/// a wrapper around Game that allows you to call Game functions from another thread
pub struct GameRunner<G: GameProvider> {
	game: Game<G>,
	rx: mpsc::Receiver<GameCommand<G>>,
}
impl<G: GameProvider> GameRunner<G> {
	pub fn new(game: Game<G>) -> (Self, mpsc::Sender<GameCommand<G>>) {
		let (tx, rx) = tokio::sync::mpsc::channel(10);

		(Self { game, rx }, tx)
	}
}
impl<G: GameProvider> Layable for GameRunner<G> {
	fn size(&self) -> (i32, i32) {
		self.game.size()
	}
	fn render(&self, d: &mut sui::Handle, det: sui::Details, scale: f32) {
		self.game.render(d, det, scale);
	}

	fn tick(&mut self) {
		loop {
			match self.rx.try_recv() {
				Ok(command) => {
					let f = command.0;
					f(&mut self.game)
				}
				Err(_) => break,
			}
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

impl<G: GameProvider> Deref for GameRunner<G> {
	type Target = Game<G>;

	fn deref(&self) -> &Self::Target {
		&self.game
	}
}
impl<G: GameProvider> DerefMut for GameRunner<G> {
	fn deref_mut(&mut self) -> &mut Self::Target {
		&mut self.game
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use game_core::GameData;

	#[test]
	fn send_test() {
		let _ = has_to_be_send::<GameRunner<GameData>>();
	}

	fn has_to_be_send<T: Send>() {}
}
