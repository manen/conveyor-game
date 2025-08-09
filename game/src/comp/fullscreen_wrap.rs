use std::{cell::RefCell, rc::Rc};

use sui::Layable;

#[derive(Clone, Debug)]
pub struct FullscreenWrap<L: Layable> {
	layable: L,
	size_store: Rc<RefCell<(i32, i32)>>,
}
impl<L: Layable> FullscreenWrap<L> {
	pub fn new(layable: L) -> Self {
		Self {
			layable,
			size_store: Default::default(),
		}
	}
}
impl<L: Layable> Layable for FullscreenWrap<L> {
	fn size(&self) -> (i32, i32) {
		let size_store = self.size_store.borrow();
		*size_store
	}
	fn render(&self, d: &mut sui::Handle, det: sui::Details, scale: f32) {
		self.layable.render(d, det, scale);
		{
			let mut size_store = self.size_store.borrow_mut();

			let width = d.get_render_width();
			let height = d.get_render_height();
			*size_store = (width, height);
		}
	}

	fn tick(&mut self) {
		self.layable.tick();
	}
	fn pass_events(
		&mut self,
		events: impl Iterator<Item = sui::core::Event>,
		det: sui::Details,
		scale: f32,
		ret_events: &mut Vec<sui::core::ReturnEvent>,
	) {
		self.layable.pass_events(events, det, scale, ret_events);
	}
}
