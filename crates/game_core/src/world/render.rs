use std::borrow::Cow;

use sui::{Color, Details, Layable, LayableExt, comp::text::Font, raylib::prelude::RaylibDraw};

use crate::{
	Tile,
	buildings::{Building, BuildingsMap},
	maps::{OrIndexed, Tilemap},
};
use textures::{TextureID, Textures};

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

	let screen_det = Details {
		aw: d.get_render_width(),
		ah: d.get_render_height(),
		..Default::default()
	};

	for x in 0..width as i32 {
		for y in 0..height as i32 {
			let draw_x = draw_x_base + (x as f32 * render_size) as i32;
			let draw_y = draw_y_base + (y as f32 * render_size) as i32;

			let (draw_x, draw_y) = (draw_x - 1, draw_y - 1);
			let render_size_i32 = render_size_i32 + 1;

			let l_det = Details {
				x: draw_x,
				y: draw_y,
				aw: render_size_i32,
				ah: render_size_i32,
			};
			if !screen_det.intersects(&l_det) {
				// skip rendering if it wouldn't make it onto the screen anyway
				continue;
			}

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

	let screen_det = Details {
		aw: d.get_render_width(),
		ah: d.get_render_height(),
		..Default::default()
	};

	const DEBUG: bool = true;

	let mut tooltip = Option::<((i32, i32), Cow<'static, str>)>::None;
	for x in 0..buildings.width() {
		for y in 0..buildings.height() {
			let draw_x = draw_x_base + (x as f32 * render_size) as i32;
			let draw_y = draw_y_base + (y as f32 * render_size) as i32;

			let (draw_x, draw_y) = (draw_x - 1, draw_y - 1);
			let render_size_i32 = render_size_i32 + 1;

			let l_det = Details {
				x: draw_x,
				y: draw_y,
				aw: render_size_i32,
				ah: render_size_i32,
			};
			if !screen_det.intersects(&l_det) {
				// skip rendering if it wouldn't make it onto the screen anyway
				continue;
			}

			let pos = (x as i32, y as i32);

			let grid_entry = buildings
				.grid_at(pos)
				.expect("we tried rendering a building that doesn't exist");
			let (scale, is_root) = match grid_entry {
				OrIndexed::Indexed { root, .. } => (2.0, *root == pos),
				_ => (1.0, true),
			};

			if is_root {
				let building = buildings
					.at(pos)
					.expect("we tried rendering a building that doesn't exist");

				if building.texture_id() != TextureID::Transparent {
					let render = building.render(textures);
					render.render(d, l_det.mul_size(scale), 1.0);

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
								format!(
									"({x}, {y})\nneeds poll: {}\n{building:#?}",
									building.needs_poll()
								)
								.into(),
							))
						}
					}
				}
			}
		}
	}

	if let Some(((draw_x, draw_y), tooltip)) = tooltip {
		Font::default().with_font(|font| {
			d.draw_text_ex(
				font,
				&tooltip,
				sui::raylib::math::Vector2::new(draw_x as _, draw_y as _),
				11 as _,
				sui::comp::text::SPACING,
				sui::Color::WHITE,
			);
		});
	}
}
