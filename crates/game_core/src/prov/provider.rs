use std::ops::Deref;

use crate::{GameData, tool::Tool};

/// provides the game \
/// this is so `Game` can have a blanket implementation over `GameProvider`s so \
/// switching to off-thread ticking or even multiplayer is easy in the future
pub trait GameProvider {
	fn data<'a>(&'a self) -> impl Deref<Target = GameData> + 'a;

	/// called on every component tick by Game
	fn standard_tick(&mut self);

	fn tool_use(&mut self, tool: &Tool, pos: (i32, i32));
}

impl GameProvider for GameData {
	fn data<'a>(&'a self) -> impl Deref<Target = GameData> + 'a {
		self
	}
	fn standard_tick(&mut self) {
		self.tick();
	}

	fn tool_use(&mut self, tool: &Tool, pos: (i32, i32)) {
		tool.r#use(self, pos);
	}
}
