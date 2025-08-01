use anyhow::anyhow;

pub mod tilemap;

#[derive(Clone, Debug)]
pub struct Map<T> {
	size: usize,
	map: Vec<Vec<T>>,
}
impl<T: Default> Map<T> {
	pub fn new_default(size: usize) -> Self {
		let map = (0..size).map(|_| (0..size).map(|_| T::default()).collect());
		let map = map.collect();

		Self { size, map }
	}
}
impl<T> Map<T> {
	/// errors if the vector sizes differ
	pub fn new(map: Vec<Vec<T>>) -> anyhow::Result<Self> {
		let len = map.len();

		for (i, inner) in map.iter().enumerate() {
			if inner.len() != len {
				return Err(anyhow!(
					"couldn't build a Map<T> from a Vec<Vec<T>>: array number {i} differs in size (expected: {len}, got: {})",
					inner.len()
				));
			}
		}

		Ok(Self { size: len, map })
	}

	pub fn take(self) -> Vec<Vec<T>> {
		self.map
	}

	pub fn at_usize(&self, (x, y): (usize, usize)) -> Option<&T> {
		if x < self.size && y < self.size {
			Some(&self.map[x][y])
		} else {
			None
		}
	}
	pub fn at_mut_usize(&mut self, (x, y): (usize, usize)) -> Option<&mut T> {
		if x < self.size && y < self.size {
			Some(&mut self.map[x][y])
		} else {
			None
		}
	}

	pub fn at(&self, (x, y): (i32, i32)) -> Option<&T> {
		if x >= 0 && y >= 0 {
			self.at_usize((x as _, y as _))
		} else {
			None
		}
	}
	pub fn at_mut(&mut self, (x, y): (i32, i32)) -> Option<&mut T> {
		if x >= 0 && y >= 0 {
			self.at_mut_usize((x as _, y as _))
		} else {
			None
		}
	}
}
