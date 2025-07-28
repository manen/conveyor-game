use sui::{
	Layable,
	raylib::{math::Vector2, prelude::RaylibDraw},
};

use crate::{
	textures::{TextureID, Textures},
	utils::Direction,
	world::{EResource, Resource, buildings::Building, render::TILE_RENDER_SIZE},
};

#[derive(Clone, Debug)]
pub struct Conveyor {
	pub dir: Direction,
	holding: Option<EResource>,
}
impl Conveyor {
	pub fn new(dir: Direction) -> Self {
		Self { dir, holding: None }
	}
}

impl Building for Conveyor {
	fn name(&self) -> std::borrow::Cow<'static, str> {
		"conveyor".into()
	}
	fn texture_id(&self) -> TextureID {
		TextureID::ConveyorTop
	}

	fn render<'a>(
		&'a self,
		textures: &'a Textures,
	) -> impl sui::Layable + Clone + std::fmt::Debug + 'a {
		#[derive(Clone, Debug)]
		struct ConveyorRenderer<'a> {
			textures: &'a Textures,
			dir: Direction,
			holding: Option<TextureID>,
		}
		impl<'a> Layable for ConveyorRenderer<'a> {
			fn size(&self) -> (i32, i32) {
				(TILE_RENDER_SIZE, TILE_RENDER_SIZE)
			}
			/// det.aw is trusted to be TILE_RENDER_SIZE scaled properly
			fn render(&self, d: &mut sui::Handle, det: sui::Details, _scale: f32) {
				let top_texture = self.textures.texture_for(TextureID::ConveyorTop);
				if let Some(top_texture) = top_texture {
					// draws the top facing texture rotated the right way

					let (x_offset, y_offset) = match self.dir {
						Direction::Top => (0, 0),
						Direction::Right => (1, 0),
						Direction::Bottom => (1, 1),
						Direction::Left => (0, 1),
					};
					let (x_offset, y_offset) = (x_offset * det.aw, y_offset * det.aw);

					let tex_det = sui::Details {
						x: det.x + x_offset,
						y: det.y + y_offset,
						..det
					};
					top_texture.render_with_rotation(d, tex_det, self.dir.degrees());
				} else {
					d.draw_rectangle(det.x, det.y, det.aw, det.ah, sui::Color::PURPLE);
				}

				if let Some(holding) = &self.holding {
					let aw = det.aw / 2;
					let ah = det.ah / 2;
					let x = det.x + aw / 2;
					let y = det.y + ah / 2;

					let holding_det = sui::Details { x, y, aw, ah };

					self.textures.render(d, holding_det, holding);
				}
			}
		}

		ConveyorRenderer {
			textures,
			dir: self.dir,
			holding: self.holding.as_ref().map(|a| a.texture_id()),
		}
	}

	fn can_receive(&self, _resource: &EResource) -> bool {
		self.holding.is_none()
	}
	fn receive(&mut self, resource: EResource) {
		if self.can_receive(&resource) {
			self.holding = Some(resource)
		}
	}

	fn needs_poll(&self) -> bool {
		self.holding.is_some()
	}
	fn resource_sample(&self, _tile_resource: Option<EResource>) -> Option<EResource> {
		self.holding.clone()
	}
	fn poll_resource(&mut self, _tile_resource: Option<EResource>) -> Option<EResource> {
		self.holding.take()
	}

	fn pass_relatives(&self) -> &'static [(i32, i32)] {
		self.dir.rel_array()
	}
}
