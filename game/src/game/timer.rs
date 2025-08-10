use std::{
	fmt::Debug,
	time::{Duration, Instant},
};

use sui::{Compatible, Layable, LayableExt};

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

// --- rendering

pub trait TimerRenderable {
	fn render<'a>(&'a self) -> impl Layable + Debug + 'static;
}
impl TimerRenderable for Timer {
	fn render<'a>(&'a self) -> impl Layable + Debug + 'static {
		let font_size = 32;

		let millis = self.target.as_millis() as i64 - self.elapsed.as_millis() as i64;
		let millis = millis.max(0); // rendering can't go into the negatives but the underlying data can

		let total_secs = millis / 1000;
		let total_mins = total_secs / 60;
		let total_hours = total_mins / 60;

		let hours = total_hours;
		let mins = total_mins - total_hours * 60;
		let secs = total_secs - total_mins * 60;

		let in_order = [hours, mins, secs].into_iter();
		let in_order = in_order.map(|num| format!("{:02}", num));
		let in_order = in_order.map(|txt| {
			let comp = sui::Text::new(txt, font_size);
			comp
		});

		// add :s and assemble
		let mut in_order = in_order.peekable();
		let mut time_div = sui::Div::empty_horizontal_with_capacity(in_order.size_hint().0);
		// add a : after every value where there's a next one coming
		loop {
			if let Some(next) = in_order.next() {
				time_div.push(next);
				if in_order.peek().is_some() {
					let colon = sui::Text::new(":", font_size);
					time_div.push(colon);
				}
			} else {
				break;
			}
		}

		let paused_ui = if self.is_paused() {
			sui::custom(sui::Text::new("PAUSED", 16).centered()).into_comp()
		} else {
			sui::Comp::Space(sui::comp::Space::new(0, 0))
		};
		let total = sui::div([
			sui::custom_only_debug(time_div.centered()).into_comp(),
			paused_ui,
		]);
		total
	}
}

// #[derive(Clone, Debug)]
// pub struct TimerRender<'a> {
// 	timer: &'a Timer,
// }
// impl<'a> Layable for TimerRender<'a> {

// }
