use crate::assets::GameAssets;

pub mod assets;
pub mod comp;
pub mod game;
pub mod levels;
pub mod scripts;
pub mod utils;

pub use game_core as world;
pub use textures;

rust_i18n::i18n!("locales", fallback = "en");

#[tokio::main]
pub async fn start_with_rt() {
	start().await;
}

fn get_locale() -> String {
	let from_args = std::env::args().nth(1);
	match from_args {
		Some(a) => {
			mklogger::println!("using locale from cli args: {a}");
			return a;
		}
		None => (),
	};

	let locale = sys_locale::get_locale();
	mklogger::println!("sys-locale reported {locale:?}");

	let locale = match locale {
		Some(a) => a.split('-').next().map(String::from),
		None => None,
	};
	let locale = locale.unwrap_or_else(|| "en".into());

	locale
}

pub async fn start() {
	println!("Hello, world!");
	// rust_i18n::set_locale("hu");

	// let locale = locale.unwrap_or_else(|| std::env::args().next().unwrap_or_else(|| "en".into()));

	#[cfg(tokio_unstable)]
	console_subscriber::init();

	let locale = get_locale();
	mklogger::println!("using locale {locale}");
	rust_i18n::set_locale(&locale);

	let (mut rl, thread) = sui_runner::rl();

	{
		let d = rl.begin_drawing(&thread);
		let mut handle = sui::Handle::new_unfocused(d, &thread);

		let assets = GameAssets::default();
		let font =
			asset_provider_font::load_font_explicit(&assets, "font.ttf", &mut handle, 32, 1.5)
				.await
				.expect("failed to font");

		game_worldgen::init_worldgen(&assets)
			.await
			.expect("failed to initialize global WorldGenerator");

		font.set_as_global();
	};

	let stage = stage_manager::Stage::from_dyn_layable(scripts::main::main().await);
	let mut ctx = sui_runner::Context::new(stage, rl, thread);

	ctx.start();

	if let Some(textures) = textures::clear_cache().await {
		std::mem::drop(textures);
	};
	mklogger::println!("bye");
}
