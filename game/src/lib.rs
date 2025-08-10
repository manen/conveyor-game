use crate::assets::GameAssets;

pub mod assets;
pub mod comp;
pub mod game;
pub mod levels;
pub mod scripts;
pub mod textures;
pub mod utils;
pub mod world;

rust_i18n::i18n!("locales", fallback = "en");

#[tokio::main]
pub async fn start_with_rt() {
	start().await;
}

pub async fn start() {
	println!("Hello, world!");
	// rust_i18n::set_locale("hu");

	let (mut rl, thread) = sui_runner::rl();

	{
		let d = rl.begin_drawing(&thread);
		let mut handle = sui::Handle::new_unfocused(d, &thread);

		let assets = GameAssets::default();
		let font =
			asset_provider_font::load_font_explicit(&assets, "font.ttf", &mut handle, 32, 1.5)
				.await
				.expect("failed to font");

		font.set_as_global();
	};

	let stage = stage_manager::Stage::from_dyn_layable(comp::main().await);
	let mut ctx = sui_runner::Context::new(stage, rl, thread);

	ctx.start();
}
