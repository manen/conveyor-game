use std::{cell::RefCell, rc::Rc};

use sui::{Color, Layable, core::Event, raylib::prelude::RaylibDraw};

pub const RECT_SIZE: i32 = 25;

#[derive(Clone, Debug)]
pub struct ShowMouse<L: Layable> {
	layable: L,
	pos: Rc<RefCell<Option<(i32, i32)>>>,
}
impl<L: Layable> ShowMouse<L> {
	pub fn new(layable: L) -> Self {
		Self {
			layable,
			pos: Default::default(),
		}
	}
}
impl<L: Layable> Layable for ShowMouse<L> {
	fn size(&self) -> (i32, i32) {
		self.layable.size()
	}
	fn render(&self, d: &mut sui::Handle, det: sui::Details, scale: f32) {
		self.layable.render(d, det, scale);

		let mut pos = self.pos.borrow_mut();
		match pos.take() {
			None => {}
			Some(pos) => {
				d.draw_rectangle(
					pos.0 - RECT_SIZE / 2,
					pos.1 - RECT_SIZE / 2,
					RECT_SIZE,
					RECT_SIZE,
					Color::RED,
				);
			}
		}
	}

	fn tick(&mut self) {
		self.layable.tick()
	}
	fn pass_events(
		&mut self,
		events: impl Iterator<Item = sui::core::Event>,
		det: sui::Details,
		scale: f32,
		ret_events: &mut Vec<sui::core::ReturnEvent>,
	) {
		let events = events.map(|event| match event {
			Event::KeyboardEvent(..) => event,
			Event::MouseEvent(m_event) => {
				let at = m_event.at();

				{
					let mut pos = self.pos.borrow_mut();
					*pos = Some(at);
				}

				event
			}
		});

		self.layable.pass_events(events, det, scale, ret_events)
	}
}
