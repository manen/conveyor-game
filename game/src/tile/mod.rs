use std::{borrow::Cow, fmt::Debug};

use sui::{Details, Handle};

pub trait Tile: Clone + Debug {
	fn name(&self) -> Cow<'static, str>;
	fn render(&self, d: &mut Handle, det: Details);
}

pub struct DynamicTile {
	/// heap pointer, allocated with std::alloc
	ptr: *mut u8,
	layout: std::alloc::Layout,
	type_name: &'static str,

	clone: fn(*const u8, layout: std::alloc::Layout) -> *mut u8,
	debug: fn(*const u8, f: &mut std::fmt::Formatter) -> std::fmt::Result,

	name: fn(*const u8) -> Cow<'static, str>,
	render: fn(*const u8, d: &mut Handle, det: Details),

	drop: fn(*mut u8),
}
impl DynamicTile {
	pub fn new<T: Tile>(tile: T) -> Self {
		// trait impls & drop
		fn clone<T: Tile>(ptr: *const u8, layout: std::alloc::Layout) -> *mut u8 {
			let ptr_borrowed: &T = unsafe { &*(ptr as *const T) };

			let new_ptr = unsafe { std::alloc::alloc(layout) };
			let new_ptr_borrowed = unsafe { &mut *(new_ptr as *mut T) };
			T::clone_into(&ptr_borrowed, new_ptr_borrowed);

			new_ptr
		}
		fn debug<T: Tile>(ptr: *const u8, f: &mut std::fmt::Formatter) -> std::fmt::Result {
			let ptr_borrowed: &T = unsafe { &*(ptr as *const T) };
			ptr_borrowed.fmt(f)
		}
		fn drop<T: Tile>(ptr: *mut u8) {
			let mut tile: std::mem::MaybeUninit<T> = std::mem::MaybeUninit::uninit();
			unsafe { std::ptr::copy_nonoverlapping(ptr as *const T, tile.as_mut_ptr(), 1) };
			unsafe { tile.assume_init_drop() };
		}

		// methods
		fn name<T: Tile>(ptr: *const u8) -> Cow<'static, str> {
			T::name(unsafe { &*(ptr as *const T) })
		}
		fn render<T: Tile>(ptr: *const u8, d: &mut Handle, det: Details) {
			T::render(unsafe { &*(ptr as *const T) }, d, det)
		}

		// allocate & move
		let type_name = std::any::type_name::<T>();
		let layout = std::alloc::Layout::new::<T>();
		let ptr = unsafe { std::alloc::alloc(layout) } as *mut T;
		unsafe {
			std::ptr::copy_nonoverlapping(&tile as *const T, ptr, 1);
		};
		std::mem::forget(tile);
		let ptr = ptr as *mut u8;

		// return
		Self {
			ptr,
			layout,
			type_name,
			clone: clone::<T>,
			debug: debug::<T>,
			name: name::<T>,
			render: render::<T>,
			drop: drop::<T>,
		}
	}
}
impl Clone for DynamicTile {
	fn clone(&self) -> Self {
		let new_ptr = (self.clone)(self.ptr, self.layout);

		let cloned = Self {
			ptr: new_ptr,
			..*self
		};
		cloned
	}
}
impl Debug for DynamicTile {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		(self.debug)(self.ptr, f)
	}
}
impl Drop for DynamicTile {
	fn drop(&mut self) {
		(self.drop)(self.ptr)
	}
}
impl Tile for DynamicTile {
	fn name(&self) -> Cow<'static, str> {
		(self.name)(self.ptr)
	}
	fn render(&self, d: &mut Handle, det: Details) {
		(self.render)(self.ptr, d, det)
	}
}
