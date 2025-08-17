use crate::GameData;

/// provides the game \
/// this is so `Game` can have a blanket implementation over `GameProvider`s so \
/// switching to off-thread ticking or even multiplayer is easy in the future
pub trait GameProvider {
	fn with_data<T, F: FnOnce(&GameData) -> T>(&self, f: F) -> T;
	fn with_data_mut<T, F: FnOnce(&mut GameData) -> T>(&mut self, f: F) -> T;
}

impl GameProvider for GameData {
	fn with_data<T, F: FnOnce(&GameData) -> T>(&self, f: F) -> T {
		f(self)
	}
	fn with_data_mut<T, F: FnOnce(&mut GameData) -> T>(&mut self, f: F) -> T {
		f(self)
	}
}
