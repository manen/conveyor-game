// GameData and GameProvider

use std::time::Duration;
pub const GAME_TICK_FREQUENCY: Duration = const {
	let millis = Duration::from_secs(1).as_millis() / 13;
	Duration::from_millis(millis as u64)
};

mod data;
pub use data::*;
mod provider;
pub use provider::*;
