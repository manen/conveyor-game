use std::{cell::RefCell, ops::Deref, rc::Rc};

use sui::{
	Color, Layable,
	raylib::{
		math::{Rectangle, Vector2},
		prelude::RaylibDraw,
	},
	tex::{LoadTextureError, Texture},
};
use tokio::sync::oneshot;

#[derive(Debug)]
pub enum AsyncTextureState {
	Loading(oneshot::Receiver<(Vec<u8>, (i32, i32))>),
	Loaded(Texture),
}
impl AsyncTextureState {
	fn tick_self(&mut self, d: &mut sui::Handle) -> Result<(), LoadTextureError> {
		let img_data = match self {
			Self::Loading(rx) => {
				let img_data = match rx.try_recv() {
					Ok(a) => a,
					Err(_) => return Ok(()),
				};
				img_data
			}
			_ => return Ok(()),
		};

		let (pixels, size) = img_data;
		let tex = Texture::new_from_rgba8(pixels, size, d)?;

		*self = Self::Loaded(tex);
		Ok(())
	}
	fn with_texture_no_update(&self) -> Option<&Texture> {
		match self {
			Self::Loaded(a) => Some(a),
			_ => None,
		}
	}
	pub fn update_self(&mut self, d: &mut sui::Handle) -> Result<(), LoadTextureError> {
		self.tick_self(d)?;
		self.with_texture_no_update();
		Ok(())
	}
	fn update_self_or_log(&mut self, d: &mut sui::Handle) {
		match self.update_self(d) {
			Ok(_) => {}
			Err(err) => {
				eprintln!("failed to load texture in AsyncTexture: {err:?}")
			}
		}
	}
}

#[derive(Debug)]
pub struct AsyncTexture {
	state: Rc<RefCell<AsyncTextureState>>,
}
impl AsyncTexture {
	pub fn from_channel(rx: oneshot::Receiver<(Vec<u8>, (i32, i32))>) -> Self {
		let state = AsyncTextureState::Loading(rx);
		let state = Rc::new(RefCell::new(state));
		AsyncTexture { state }
	}
}
impl AsyncTexture {
	/// doesn't update itself
	fn with_texture_no_update<T, F: FnOnce(&Texture) -> T>(&self, f: F) -> Option<T> {
		let state = self.state.borrow();
		let tex = state.with_texture_no_update();
		tex.map(f)
	}
	/// updates itself
	pub fn update_self(&self, d: &mut sui::Handle) -> Result<(), LoadTextureError> {
		let mut state = self.state.borrow_mut();
		state.update_self(d)
	}
	fn update_self_or_log(&self, d: &mut sui::Handle) {
		match self.update_self(d) {
			Ok(_) => {}
			Err(err) => {
				eprintln!("failed to load texture in AsyncTexture: {err:?}")
			}
		}
	}
	/// attempts to update itself, before executing the closure if loaded
	pub fn with_texture<T, F: FnOnce(&Texture) -> T>(
		&self,
		d: &mut sui::Handle,
		f: F,
	) -> Option<T> {
		let mut state = self.state.borrow_mut();
		state.update_self_or_log(d);
		state.with_texture_no_update().map(f)
	}

	pub fn size(&self) -> (i32, i32) {
		self.with_texture_no_update(|tex| tex.size())
			.unwrap_or_default()
	}
	pub fn width(&self) -> i32 {
		self.with_texture_no_update(|tex| tex.width())
			.unwrap_or_default()
	}
	pub fn height(&self) -> i32 {
		self.with_texture_no_update(|tex| tex.height())
			.unwrap_or_default()
	}

	pub fn render(&self, d: &mut sui::Handle, det: sui::Details) {
		self.render_with_rotation(d, det, 0.0);
	}
	/// does not correct for the position change caused by the rotation
	pub fn render_with_rotation(&self, d: &mut sui::Handle, det: sui::Details, degrees: f32) {
		self.update_self_or_log(d);
		self.with_texture_no_update(|tex| {
			d.draw_texture_pro(
				tex.as_ref(),
				Rectangle {
					x: 0.0,
					y: 0.0,
					width: tex.width() as _,
					height: tex.height() as _,
				},
				Rectangle {
					x: det.x as _,
					y: det.y as _,
					width: det.aw as _,
					height: det.ah as _,
				},
				Vector2::default(),
				degrees,
				Color::new(255, 255, 255, 255),
			);
		});
	}
}
impl Layable for AsyncTexture {
	fn size(&self) -> (i32, i32) {
		(self.width(), self.height())
	}
	fn render(&self, d: &mut sui::Handle, det: sui::Details, scale: f32) {
		let det = det.mul_size(scale);
		// regular self.render already takes care of loading and everything
		self.render(d, det);
		// d.draw_rectangle_lines(det.x, det.y, det.aw, det.ah, Color::RED);
	}
}

/// creates a new AsyncTexture and schedules its loading on another tokio task
pub fn background_loaded<F: Future<Output = (Vec<u8>, (i32, i32))> + Send + 'static>(
	loader: F,
) -> AsyncTexture {
	let (tx, rx) = oneshot::channel();

	tokio::spawn(async move {
		let out = loader.await;
		tx.send(out)
	});

	AsyncTexture::from_channel(rx)
}
