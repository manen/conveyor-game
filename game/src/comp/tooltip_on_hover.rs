use std::{borrow::Cow, cell::RefCell, rc::Rc};

use sui::{
	Layable,
	raylib::{math::glam::Vec2, prelude::RaylibDraw},
};

pub type TooltipData = Rc<RefCell<Option<((i32, i32), Cow<'static, str>)>>>;

#[derive(Clone, Debug)]
/// parent component of TooltipOnHover. this is what actually renders the tooltip, after
/// all the children are already rendered, so the text is always on top
pub struct TooltipProvider<L: Layable> {
	container: L,
	data: TooltipData,
}
impl<L: Layable> TooltipProvider<L> {
	pub fn new_explicit(layable: L, data: TooltipData) -> Self {
		Self {
			container: layable,
			data,
		}
	}

	/// the easiest way to make tooltips since you just clone the TooltipData in all of the TooltipOnHovers and you're good
	pub fn new<F: FnOnce(&TooltipData) -> L>(f: F) -> Self {
		let data = TooltipData::default();
		let layable = f(&data);

		Self::new_explicit(layable, data)
	}
}
impl<L: Layable> Layable for TooltipProvider<L> {
	fn size(&self) -> (i32, i32) {
		self.container.size()
	}
	fn render(&self, d: &mut sui::Handle, det: sui::Details, scale: f32) {
		self.container.render(d, det, scale);

		{
			let mut handle = self.data.borrow_mut();
			if let Some((pos, tooltip)) = handle.take() {
				d.draw_text(&tooltip, pos.0, pos.1, 24, sui::Color::WHITE);
			}
		}
	}

	fn tick(&mut self) {
		self.container.tick();
	}
	fn pass_events(
		&mut self,
		events: impl Iterator<Item = sui::core::Event>,
		det: sui::Details,
		scale: f32,
		ret_events: &mut Vec<sui::core::ReturnEvent>,
	) {
		self.container.pass_events(events, det, scale, ret_events);
	}
}

#[derive(Clone, Debug)]
/// these do nothing on their own, there needs to be a [TooltipProvider] up in the component tree
pub struct TooltipOnHover<L: Layable> {
	layable: L,
	tooltip: Cow<'static, str>,
	data: TooltipData,
}
impl<L: Layable> TooltipOnHover<L> {
	pub fn new(tooltip: impl Into<Cow<'static, str>>, data: TooltipData, layable: L) -> Self {
		let tooltip = tooltip.into();
		Self {
			layable,
			data,
			tooltip,
		}
	}
}
impl<L: Layable> Layable for TooltipOnHover<L> {
	fn size(&self) -> (i32, i32) {
		self.layable.size()
	}
	fn render(&self, d: &mut sui::Handle, det: sui::Details, scale: f32) {
		self.layable.render(d, det, scale);

		let Vec2 { x, y } = d.get_mouse_position();
		let (x, y) = (x as i32, y as i32);

		if det.is_inside(x, y) {
			// d.draw_text(&self.tooltip, x, y, 24, sui::Color::WHITE);
			let mut data = self.data.borrow_mut();
			*data = Some(((x, y), self.tooltip.clone()))
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
