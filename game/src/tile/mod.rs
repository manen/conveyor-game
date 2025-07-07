use std::{borrow::Cow, fmt::Debug};
use sui::{Details, Handle};

mod r#dyn;
pub use r#dyn::DynamicTile;

pub mod tiles;
pub use tiles::*;

pub trait Tile: Clone + Debug {
	fn name(&self) -> Cow<'static, str>;
	fn render(&self, d: &mut Handle, det: Details);
}
