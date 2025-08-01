use anyhow::anyhow;

mod tilemap;
pub use tilemap::*;

#[derive(Clone, Debug)]
pub struct Map<T> {
	width: usize,
	height: usize,
	map: Vec<Vec<T>>,
}
impl<T: Default> Map<T> {
	pub fn new_default(width: usize, height: usize) -> Self {
		let map = (0..width).map(|_| (0..height).map(|_| T::default()).collect());
		let map = map.collect();

		Self { width, height, map }
	}
}
impl<T> Map<T> {
	/// errors if the vector sizes differ
	pub fn from_vec(map: Vec<Vec<T>>) -> anyhow::Result<Self> {
		let width = map.len();

		let mut height = None;
		for (i, inner) in map.iter().enumerate() {
			if let Some(height) = height {
				if inner.len() != height {
					return Err(anyhow!(
						"couldn't build a Map<T> from a Vec<Vec<T>>: array number {i} differs in size (expected: {height:?}, got: {})",
						inner.len()
					));
				}
			} else {
				height = Some(inner.len())
			}
		}

		if let Some(height) = height {
			Ok(Self { width, height, map })
		} else {
			Err(anyhow!("attempted to create Map<T> from empty array"))
		}
	}

	pub fn take(self) -> Vec<Vec<T>> {
		self.map
	}

	pub fn at_usize(&self, (x, y): (usize, usize)) -> Option<&T> {
		if x < self.width && y < self.height {
			Some(&self.map[x][y])
		} else {
			None
		}
	}
	pub fn at_mut_usize(&mut self, (x, y): (usize, usize)) -> Option<&mut T> {
		if x < self.width && y < self.height {
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

	pub fn iter_inner(&self) -> impl Iterator<Item = &Vec<T>> {
		self.map.iter()
	}

	pub fn iter(&self) -> impl Iterator<Item = ((i32, i32), &T)> {
		self.iter_coords().map(|coords| (coords, self.at(coords).expect("Map<T>::iter_coords and Map<T>::iter are wrong! coordinate returned by Map<T>::iter_coords returned a coordinate that self.at() returned None for")))
	}
	pub fn iter_coords_usize(&self) -> impl Iterator<Item = (usize, usize)> {
		(0..self.width)
			.map(move |x| (0..self.height).map(move |y| (x, y)))
			.flatten()
	}
	pub fn iter_coords(&self) -> impl Iterator<Item = (i32, i32)> {
		self.iter_coords_usize().map(|(x, y)| (x as _, y as _))
	}
}
