use std::borrow::Cow;

use strum::{EnumIter, IntoEnumIterator};

/// an enum for every texture we can use \
/// extensible in the future by adding an Other(u64) and have some sort of setup that hashes their
/// resource_path, just so it's faster \
/// this id is used a lot in every tick
#[derive(Clone, Debug, PartialEq, Eq, Hash, EnumIter)]
pub enum TextureID {
	Transparent,
	Eraser,

	SmallExtractor,
	ChannelConsumer,

	Stone,
	IronOre,
	CoalOre,

	Coal,
	RawIron,

	ConveyorTop,
}
impl TextureID {
	/// none just becomes transparent
	pub const fn resource_path(&self) -> Cow<'static, str> {
		match self {
			TextureID::Transparent => Cow::Borrowed("textures/transparent.png"),
			TextureID::Eraser => Cow::Borrowed("textures/eraser.png"),

			TextureID::SmallExtractor => Cow::Borrowed("textures/small-extractor.png"),
			TextureID::ChannelConsumer => Cow::Borrowed("textures/channel-consumer.png"),

			TextureID::Stone => Cow::Borrowed("textures/stone.png"),
			TextureID::IronOre => Cow::Borrowed("textures/iron_ore.png"),
			TextureID::CoalOre => Cow::Borrowed("textures/coal_ore.png"),
			TextureID::Coal => Cow::Borrowed(
				"https://static.wikia.nocookie.net/minecraft/images/a/a7/Coal.png/revision/latest/scale-to-width/360?cb=20200814153155",
			),
			TextureID::RawIron => Cow::Borrowed(
				"https://static.wikia.nocookie.net/minecraft_gamepedia/images/d/d2/Raw_Iron_JE3_BE2.png/revision/latest?cb=20210421181435",
			),
			TextureID::ConveyorTop => Cow::Borrowed("textures/conveyor-top.png"),
		}
	}
}

pub fn all_textures() -> impl Iterator<Item = TextureID> {
	// &[
	// 	TextureID::Stone,
	// 	TextureID::IronOre,
	// 	TextureID::CoalOre,
	// 	TextureID::RawIron,
	// ]
	TextureID::iter()
}
