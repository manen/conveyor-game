use std::ops::Deref;

use crate::GameData;

/// provides the game \
/// this is so `Game` can have a blanket implementation over `GameProvider`s so \
/// switching to off-thread ticking or even multiplayer is easy in the future
pub trait GameProvider {
	fn data<'a>(&'a self) -> impl Deref<Target = GameData> + 'a;

	// fn tool_use(&mut self, tool)
}
