use sui::{Details, Layable, core::Event};

#[derive(Clone, Debug)]
/// a container over two elements, laid out horizontally.
/// the first one gets all the space it requested, the second gets the remainder
pub struct Two<A: Layable, B: Layable> {
	left: A,
	right: B,
}
impl<A: Layable, B: Layable> Two<A, B> {
	pub fn new(left: A, right: B) -> Self {
		Self { left, right }
	}

	fn dets(&self, det: Details, scale: f32) -> (Details, Details) {
		let (l_w, l_h) = self.left.size();
		let l_det = Details {
			aw: (l_w as f32 * scale) as i32,
			ah: (l_h as f32 * scale) as i32,
			..det
		};

		let remainder = Details {
			x: det.x + l_w,
			y: det.y,
			aw: (det.aw as f32 * scale) as i32 - l_det.aw,
			ah: (det.ah as f32 * scale) as i32 - l_det.ah,
		};

		(l_det, remainder)
	}
}
impl<A: Layable, B: Layable> Layable for Two<A, B> {
	fn size(&self) -> (i32, i32) {
		let (l_w, l_h) = self.left.size();
		let (r_w, r_h) = self.right.size();

		(l_w + r_w, l_h.max(r_h))
	}
	fn render(&self, d: &mut sui::Handle, det: Details, scale: f32) {
		let (l_det, r_det) = self.dets(det, scale);
		self.left.render(d, l_det, scale);
		self.right.render(d, r_det, scale);
	}

	fn tick(&mut self) {
		self.left.tick();
		self.right.tick();
	}
	fn pass_events(
		&mut self,
		events: impl Iterator<Item = sui::core::Event>,
		det: sui::Details,
		scale: f32,
		ret_events: &mut Vec<sui::core::ReturnEvent>,
	) {
		let (l_det, r_det) = self.dets(det, scale);

		for event in events {
			match event {
				Event::KeyboardEvent(..) => {
					self.left
						.pass_events(std::iter::once(event), l_det, scale, ret_events);
					self.right
						.pass_events(std::iter::once(event), r_det, scale, ret_events);
				}
				Event::MouseEvent(m_event) => {
					if l_det.is_inside_tuple(m_event.at()) {
						self.left
							.pass_events(std::iter::once(event), l_det, scale, ret_events);
					} else {
						self.right
							.pass_events(std::iter::once(event), r_det, scale, ret_events);
					}
				}
			}
		}
	}
}
