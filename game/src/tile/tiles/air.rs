use crate::tile::Tile;

#[derive(Copy, Clone, Debug)]
pub struct Air;
impl Tile for Air {
	fn name(&self) -> std::borrow::Cow<'static, str> {
		"air".into()
	}
	fn render(&self, _d: &mut sui::Handle, _det: sui::Details) {}
}
