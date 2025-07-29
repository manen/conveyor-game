use sui::{Layable, core::ReturnEvent};

mod multimap;
pub use multimap::MultiMap;

/// a layable that'll indiscriminantly return every event it's passed
#[derive(Clone, Debug)]
pub struct ReturnEvents;
impl Layable for ReturnEvents {
	fn size(&self) -> (i32, i32) {
		(0, 0)
	}
	fn render(&self, _d: &mut sui::Handle, _det: sui::Details, _scale: f32) {}

	fn pass_event(
		&mut self,
		event: sui::core::Event,
		_det: sui::Details,
		_scale: f32,
	) -> Option<sui::core::ReturnEvent> {
		Some(ReturnEvent::new(event))
	}
}

#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug)]
pub enum Direction {
	Right,
	Bottom,
	Left,
	Top,
}
impl Direction {
	/// clockwise
	pub const fn rotate_r(self) -> Self {
		match self {
			Direction::Right => Direction::Bottom,
			Direction::Bottom => Direction::Left,
			Direction::Left => Direction::Top,
			Direction::Top => Direction::Right,
		}
	}
	pub const fn rotate_l(self) -> Self {
		match self {
			Direction::Right => Direction::Top,
			Direction::Bottom => Direction::Right,
			Direction::Left => Direction::Bottom,
			Direction::Top => Direction::Left,
		}
	}
	pub const fn reverse(self) -> Self {
		match self {
			Direction::Right => Direction::Left,
			Direction::Bottom => Direction::Top,
			Direction::Left => Direction::Right,
			Direction::Top => Direction::Bottom,
		}
	}

	pub fn is_axis_same(&self, b: &Self) -> bool {
		b == self || b == &self.reverse()
	}

	pub const fn degrees_i32(self) -> i32 {
		match self {
			Direction::Top => 0,
			Direction::Right => 90,
			Direction::Bottom => 180,
			Direction::Left => 270,
		}
	}
	pub const fn degrees(self) -> f32 {
		self.degrees_i32() as f32
	}

	pub const fn rel(self) -> (i32, i32) {
		self.rel_mul(1)
	}
	pub const fn from_rel(rel: (i32, i32)) -> Option<Self> {
		match rel {
			(1, 0) => Some(Self::Right),
			(0, 1) => Some(Self::Bottom),
			(-1, 0) => Some(Self::Left),
			(0, -1) => Some(Self::Top),
			_ => None,
		}
	}

	pub const fn rel_array(self) -> &'static [(i32, i32)] {
		match self {
			Self::Top => &[(0, -1)],
			Self::Bottom => &[(0, 1)],
			Self::Left => &[(-1, 0)],
			Self::Right => &[(1, 0)],
		}
	}

	pub const fn rel_mul(self, mul: i32) -> (i32, i32) {
		match self {
			Self::Right => (mul, 0),
			Self::Bottom => (0, mul),
			Self::Left => (-mul, 0),
			Self::Top => (0, -mul),
		}
	}

	pub fn all() -> impl Iterator<Item = Self> {
		[Self::Right, Self::Bottom, Self::Left, Self::Top].into_iter()
	}
}
