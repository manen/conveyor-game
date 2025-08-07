use std::borrow::Cow;

use sui::{Color, Details, Layable, raylib::prelude::RaylibDraw};

use crate::{
	textures::{TextureID, Textures},
	world::{
		Tile,
		buildings::{Building, BuildingsMap},
		maps::Tilemap,
	},
};

pub const TILE_RENDER_SIZE: i32 = 32;

pub fn draw_tilemap(
	d: &mut sui::Handle,
	tilemap: &Tilemap,
	textures: &Textures,
	draw_x_base: i32,
	draw_y_base: i32,
	scale: f32,

	// map size
	width: usize,
	height: usize,
) {
	let render_size = TILE_RENDER_SIZE as f32 * scale;
	let render_size_i32 = render_size as i32;

	for x in 0..width as i32 {
		for y in 0..height as i32 {
			let draw_x = draw_x_base + (x as f32 * render_size) as i32;
			let draw_y = draw_y_base + (y as f32 * render_size) as i32;

			let (draw_x, draw_y) = (draw_x - 1, draw_y - 1);
			let render_size_i32 = render_size_i32 + 1;

			let tile = tilemap
				.at((x, y))
				.expect("we tried rendering a tile that doesn't exist");

			// let name = tile.name();
			// d.draw_text(&name, draw_x, draw_y, 11, sui::Color::WHITE);

			let tiletex = tile.texture_id();

			let tex = textures.texture_for(tiletex);
			match tex {
				None => {
					d.draw_rectangle(
						draw_x,
						draw_y,
						render_size_i32,
						render_size_i32,
						Color::PURPLE,
					);
				}
				Some(tex) => {
					tex.render(
						d,
						Details::new(draw_x, draw_y, render_size_i32, render_size_i32),
					);
				}
			}
		}
	}
}

pub fn draw_buildings(
	d: &mut sui::Handle,
	buildings: &BuildingsMap,
	textures: &Textures,
	draw_x_base: i32,
	draw_y_base: i32,
	scale: f32,
) {
	let render_size = TILE_RENDER_SIZE as f32 * scale;
	let render_size_i32 = render_size as i32;

	const DEBUG: bool = true;

	let mut tooltip = Option::<((i32, i32), Cow<'static, str>)>::None;
	for x in 0..buildings.width() {
		for y in 0..buildings.height() {
			let draw_x = draw_x_base + (x as f32 * render_size) as i32;
			let draw_y = draw_y_base + (y as f32 * render_size) as i32;

			let (draw_x, draw_y) = (draw_x - 1, draw_y - 1);
			let render_size_i32 = render_size_i32 + 1;

			let building = buildings
				.at((x as _, y as _))
				.expect("we tried rendering a building that doesn't exist");

			if building.texture_id() != TextureID::Transparent {
				let render = building.render(textures);
				let l_det = Details {
					x: draw_x,
					y: draw_y,
					aw: render_size_i32,
					ah: render_size_i32,
				};
				render.render(d, l_det, 1.0);

				if DEBUG {
					let cursor_inside = Details {
						x: draw_x,
						y: draw_y,
						aw: render_size_i32,
						ah: render_size_i32,
					}
					.is_inside(d.get_mouse_x(), d.get_mouse_y());
					if cursor_inside {
						tooltip = Some((
							(draw_x, draw_y),
							format!("({x}, {y})\n{building:?}\n{}", building.needs_poll()).into(),
						))
					}
				}
			}
		}
	}

	if let Some(((draw_x, draw_y), tooltip)) = tooltip {
		d.draw_text(&tooltip, draw_x, draw_y, 11, sui::Color::WHITE);
	}
}
