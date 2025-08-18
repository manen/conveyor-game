// GameData and GameProvider

use std::time::Duration;
pub const GAME_TICK_FREQUENCY: Duration = Duration::from_millis(1000 / 20);

mod data;
pub use data::*;
mod provider;
pub use provider::*;
