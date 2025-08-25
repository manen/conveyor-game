use std::sync::Arc;

use asset_provider::Assets;
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
				mklogger::eprintln!("failed to load texture in AsyncTexture: {err:?}")
			}
		}
	}
}

#[derive(Debug)]
/// a texture variant that is valid uninitialized, can be initialized by \
/// sending the image data though a tokio oneshot channel.
/// initialized on the first render call where the image data can be received
pub struct AsyncTexture {
	state: Arc<std::sync::Mutex<AsyncTextureState>>,
}
impl AsyncTexture {
	pub fn from_channel(rx: oneshot::Receiver<(Vec<u8>, (i32, i32))>) -> Self {
		let state = AsyncTextureState::Loading(rx);
		// let state = Rc::new(RefCell::new(state));
		let state = Arc::new(std::sync::Mutex::new(state));
		AsyncTexture { state }
	}
}
impl AsyncTexture {
	/// doesn't update itself
	fn with_texture_no_update<T, F: FnOnce(&Texture) -> T>(&self, f: F) -> Option<T> {
		let state = self.state.try_lock().ok()?;
		let tex = state.with_texture_no_update();
		tex.map(f)
	}
	/// updates itself
	pub fn update_self(&self, d: &mut sui::Handle) -> Result<(), LoadTextureError> {
		let mut state = match self.state.try_lock() {
			Ok(a) => a,
			Err(err) => {
				mklogger::eprintln!("failed to lock AsyncTexture state Mutex: {err:?}");
				return Ok(());
			}
		};
		state.update_self(d)
	}
	fn update_self_or_log(&self, d: &mut sui::Handle) {
		match self.update_self(d) {
			Ok(_) => {}
			Err(err) => {
				mklogger::eprintln!("failed to load texture in AsyncTexture: {err:?}")
			}
		}
	}
	/// attempts to update itself, before executing the closure if loaded
	pub fn with_texture<T, F: FnOnce(&Texture) -> T>(
		&self,
		d: &mut sui::Handle,
		f: F,
	) -> Option<T> {
		let mut state = self.state.try_lock().ok()?;
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
		let _ = tx.send(out);
	});

	AsyncTexture::from_channel(rx)
}

/// creates a new AsyncTexture with its image data already loaded, so it will be
/// loaded into a Texture as soon as the first render call is made
pub fn from_rgba8(pixels: Vec<u8>, size: (i32, i32)) -> AsyncTexture {
	let (tx, rx) = oneshot::channel();
	let _ = tx.send((pixels, size));

	AsyncTexture::from_channel(rx)
}

use asset_provider_image::AssetsExt;

/// only returns once the image has been loaded and converted into rgba8
pub async fn from_asset<A: Assets + Send + Sync>(
	assets: &A,
	key: &str,
) -> anyhow::Result<AsyncTexture> {
	let image = assets.asset_image(key).await?;
	let rgba = image.to_rgba8();

	let (w, h) = rgba.dimensions();
	let pixels = rgba.into_raw();
	let size = (w as i32, h as i32);

	Ok(from_rgba8(pixels, size))
}

#[cfg(test)]
mod tests {
	use super::*;

	fn has_to_be_send<T: Send>() {}
	#[test]
	fn test_send() {
		has_to_be_send::<AsyncTexture>();
	}
}

// ennek ugye az a lenyege hogy legyen a tutorialban kis preview hogy hogy kene kineznie a kovi lepesnek
// pl a kemence bekotesnel ez geci fontos lenne hogy valami vizualis cue is legyen
