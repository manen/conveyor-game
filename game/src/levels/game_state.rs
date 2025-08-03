use std::path::PathBuf;

use anyhow::Context;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct GameState {
	pub tutorial_completed: bool,
}

impl GameState {
	pub fn get_fs_path() -> anyhow::Result<PathBuf> {
		let filename = "conveyor-game.cgstate";

		if !cfg!(debug_assertions) {
			let home =
				dirs::home_dir().with_context(|| format!("failed to get user's home directory"))?;

			Ok(home.join(filename))
		} else {
			Ok(std::env::current_dir()?.join(filename))
		}
	}

	pub async fn load_with_error() -> anyhow::Result<Self> {
		let path = Self::get_fs_path()?;
		let file = tokio::fs::OpenOptions::new()
			.read(true)
			.open(&path)
			.await
			.with_context(|| format!("while reading GameState from {}", path.display()))?;
		let mut file = file.into_std().await;

		let decoded: GameState =
			bincode::serde::decode_from_std_read(&mut file, bincode::config::standard())
				.with_context(|| format!("while decoding GameState from {}", path.display()))?;

		Ok(decoded)
	}
	pub async fn load() -> Self {
		let with_error = Self::load_with_error().await;
		match with_error {
			Ok(a) => a,
			Err(err) => {
				eprintln!("failed to load GameState:\n{err}");
				Self::default()
			}
		}
	}

	pub async fn save(&self) -> anyhow::Result<()> {
		let path = Self::get_fs_path()?;
		let file = tokio::fs::OpenOptions::new()
			.write(true)
			.create(true)
			.open(&path)
			.await
			.with_context(|| {
				format!(
					"while opening the file we want to write GameState into ({})",
					path.display()
				)
			})?;
		let mut file = file.into_std().await;

		bincode::serde::encode_into_std_write(self, &mut file, bincode::config::standard())?;
		Ok(())
	}
}
