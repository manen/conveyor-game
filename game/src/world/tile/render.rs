use sui::{Color, Details, raylib::prelude::RaylibDraw};

use crate::world::{
	Tile,
	tilemap::{SIZE, Tilemap},
};

pub const TILE_RENDER_SIZE: i32 = 32;

pub fn draw_tilemap(
	d: &mut sui::Handle,
	tilemap: &Tilemap,
	draw_x_base: i32,
	draw_y_base: i32,
	scale: f32,
) {
	let render_size = TILE_RENDER_SIZE as f32 * scale;
	let render_size_i32 = render_size as i32;

	for x in 0..SIZE {
		for y in 0..SIZE {
			let draw_x = draw_x_base + (x as f32 * render_size) as i32;
			let draw_y = draw_y_base + (y as f32 * render_size) as i32;

			let tile = tilemap
				.at((x, y))
				.expect("we tried rendering a tile that doesn't exist");

			// let name = tile.name();
			// d.draw_text(&name, draw_x, draw_y, 11, sui::Color::WHITE);

			let tiletex = tile.tile_texture_id();

			let tex = tilemap.texture_for(tiletex);
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
