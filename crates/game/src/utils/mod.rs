use sui::{Layable, core::ReturnEvent};

mod show_mouse;
pub use show_mouse::ShowMouse;

mod connected;
pub use connected::*;

/// a layable that'll indiscriminantly return every event it's passed
#[derive(Clone, Debug)]
pub struct ReturnEvents;
impl Layable for ReturnEvents {
	fn size(&self) -> (i32, i32) {
		(0, 0)
	}
	fn render(&self, _d: &mut sui::Handle, _det: sui::Details, _scale: f32) {}

	fn pass_events(
		&mut self,
		events: impl Iterator<Item = sui::core::Event>,
		_det: sui::Details,
		_scale: f32,
		ret_events: &mut Vec<ReturnEvent>,
	) {
		ret_events.extend(events.map(ReturnEvent::new))
	}
}
