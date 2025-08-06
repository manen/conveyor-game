use std::borrow::Cow;

use sui::{
	Layable,
	raylib::{math::glam::Vec2, prelude::RaylibDraw},
};

#[derive(Clone, Debug)]
pub struct TooltipOnHover<L: Layable> {
	layable: L,
	tooltip: Cow<'static, str>,
}
impl<L: Layable> TooltipOnHover<L> {
	pub fn new(tooltip: impl Into<Cow<'static, str>>, layable: L) -> Self {
		let tooltip = tooltip.into();
		Self { layable, tooltip }
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
			d.draw_text(&self.tooltip, x, y, 24, sui::Color::WHITE);
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
