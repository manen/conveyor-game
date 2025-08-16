use std::{
	fmt::Debug,
	ops::{Deref, DerefMut},
};

#[derive(Copy, Clone, PartialEq, Eq, Hash, Default)]
pub struct NoDebug<T> {
	val: T,
}
impl<T> NoDebug<T> {
	pub fn new(val: T) -> Self {
		Self { val }
	}

	pub fn take(self) -> T {
		self.val
	}
}

impl<T> Debug for NoDebug<T> {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(f, "_")
	}
}

impl<T> AsRef<T> for NoDebug<T> {
	fn as_ref(&self) -> &T {
		&self.val
	}
}
impl<T> AsMut<T> for NoDebug<T> {
	fn as_mut(&mut self) -> &mut T {
		&mut self.val
	}
}
impl<T> Deref for NoDebug<T> {
	type Target = T;

	fn deref(&self) -> &Self::Target {
		&self.val
	}
}
impl<T> DerefMut for NoDebug<T> {
	fn deref_mut(&mut self) -> &mut Self::Target {
		&mut self.val
	}
}

impl<T> From<T> for NoDebug<T> {
	fn from(value: T) -> Self {
		Self::new(value)
	}
}
