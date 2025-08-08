use std::time::{Duration, Instant};

#[derive(Clone, Debug)]
pub enum Pause {
	Paused,
	Ticking { last_tick: Instant },
}

#[derive(Clone, Debug)]
pub struct Timer {
	target: Duration,
	elapsed: Duration,
	pause: Pause,
}
impl Timer {
	pub fn new(target: Duration) -> Self {
		Self {
			target,
			elapsed: Duration::default(),
			pause: Pause::Paused,
		}
	}

	pub fn pause(&mut self) {
		self.pause = Pause::Paused;
	}
	pub fn is_paused(&self) -> bool {
		match self.pause {
			Pause::Paused => true,
			_ => false,
		}
	}
	pub fn resume(&mut self) {
		self.pause = Pause::Ticking {
			last_tick: Instant::now(),
		};
	}
	pub fn is_ticking(&self) -> bool {
		match self.pause {
			Pause::Ticking { .. } => true,
			_ => false,
		}
	}

	pub fn tick(&mut self) {
		match &mut self.pause {
			Pause::Paused => {}
			Pause::Ticking { last_tick } => {
				let now = Instant::now();
				self.elapsed += last_tick.elapsed();
				*last_tick = now;
			}
		}
	}

	/// you need to be calling timer.tick() for this to work
	pub fn is_finished(&self) -> bool {
		self.elapsed > self.target
	}
}
