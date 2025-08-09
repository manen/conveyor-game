use std::{
	fmt::Debug,
	ops::{Deref, DerefMut},
};

use sui::Layable;
use tokio::sync::{mpsc, oneshot};

use super::Game;

pub struct GameCommand(pub Box<dyn FnOnce(&mut Game) + Send>);
impl GameCommand {
	pub fn new<F: FnOnce(&mut Game) + Send + 'static>(f: F) -> Self {
		Self(Box::new(f))
	}

	pub fn new_return<R: Debug + Send + 'static, F: FnOnce(&mut Game) -> R + Send + 'static>(
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

impl Deref for GameRunner {
	type Target = Game;

	fn deref(&self) -> &Self::Target {
		&self.game
	}
}
impl DerefMut for GameRunner {
	fn deref_mut(&mut self) -> &mut Self::Target {
		&mut self.game
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn send_test() {
		let _ = has_to_be_send::<GameRunner>();
	}

	fn has_to_be_send<T: Send>() {}
}
