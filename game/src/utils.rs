use sui::{Layable, core::ReturnEvent};

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
