use game_core::GameProvider;

use crate::{
	game::{Game, GameData},
	world::buildings::Building,
};

pub trait Target: Clone {
	fn is_finished(&self, source: (i32, i32)) -> bool;
}
impl Target for (i32, i32) {
	fn is_finished(&self, source: (i32, i32)) -> bool {
		*self == source
	}
}
impl Target for &[(i32, i32)] {
	fn is_finished(&self, source: (i32, i32)) -> bool {
		self.contains(&source)
	}
}

pub const DEBUG: bool = false;

#[derive(Clone, Debug)]
pub enum ConnectionError {
	NoBuilding { pos: (i32, i32), iterations: i32 },
}

/// connection checker utility for Game;
/// check if a block can output into another block
pub trait CheckConnection {
	type Cache;

	fn is_connected<T: Target>(
		&self,
		source: (i32, i32),
		target: T,
	) -> Result<bool, ConnectionError>;
	fn is_connected_internal<T: Target>(
		&self,
		source: (i32, i32),
		target: T,
		cache: &mut Self::Cache,
	) -> Result<bool, ConnectionError>;
}
impl CheckConnection for GameData {
	/// stores where we've been before to:
	/// - avoid logical recursion
	/// - immediately return false if we've calculated a position to be false before
	type Cache = Vec<(i32, i32)>;

	fn is_connected<T: Target>(
		&self,
		source: (i32, i32),
		target: T,
	) -> Result<bool, ConnectionError> {
		let mut cache = Default::default();
		self.is_connected_internal(source, target, &mut cache)
	}

	fn is_connected_internal<T: Target>(
		&self,
		source: (i32, i32),
		target: T,
		cache: &mut Self::Cache,
	) -> Result<bool, ConnectionError> {
		let mut source = source;

		for i in 0.. {
			// println!("checking {source:?}");

			if cache.contains(&source) {
				// we've been here before and it's false
				//     OR
				// we've been here before and we're going in a circle
				return Ok(false);
			}
			if target.is_finished(source) {
				return Ok(true);
			}

			let checking =
				self.buildings
					.at(source)
					.ok_or_else(|| ConnectionError::NoBuilding {
						pos: source,
						iterations: i,
					})?;
			cache.push(source);

			let relatives = checking.pass_directions();
			match relatives.len() {
				0 => return Ok(false),
				1 => {
					let dir = relatives[0];
					let rel = dir.rel();
					let pos = (source.0 + rel.0, source.1 + rel.1);
					source = pos;
					// println!("single direction building passing to {pos:?}");

					continue;
				}
				_ => {
					// we have to do recursion
					for dir in relatives.iter().copied() {
						let pos = dir.rel();
						let pos = (source.0 + pos.0, source.1 + pos.1);
						// println!("multireturn building passing to {pos:?}");

						let is_connected =
							match self.is_connected_internal(pos, target.clone(), cache) {
								Ok(a) => a,
								Err(err) => {
									if DEBUG {
										eprintln!(
											"dropping error in CheckConnection branch: {err:?}"
										)
									}
									false
								}
							};
						if is_connected {
							return Ok(true);
						} else {
							// cache.push(pos)
						}
					}
					// cache.push(source);
					return Ok(false);
				}
			}
		}
		// cache.push(source);
		Ok(false)
	}
}
impl<G: GameProvider> CheckConnection for Game<G> {
	type Cache = Vec<(i32, i32)>;

	fn is_connected<T: Target>(
		&self,
		source: (i32, i32),
		target: T,
	) -> Result<bool, ConnectionError> {
		self.data().is_connected(source, target)
	}

	fn is_connected_internal<T: Target>(
		&self,
		source: (i32, i32),
		target: T,
		cache: &mut Self::Cache,
	) -> Result<bool, ConnectionError> {
		self.data().is_connected_internal(source, target, cache)
	}
}
