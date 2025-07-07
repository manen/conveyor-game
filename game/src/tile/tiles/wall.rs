use sui::raylib::prelude::RaylibDraw;

use crate::tile::Tile;

#[derive(Copy, Clone, Debug)]
pub struct Wall;
impl Tile for Wall {
	fn name(&self) -> std::borrow::Cow<'static, str> {
		"wall".into()
	}
	fn render(&self, d: &mut sui::Handle, det: sui::Details) {
		d.draw_rectangle(det.x, det.y, det.aw, det.ah, sui::Color::WHITE);
	}
}
