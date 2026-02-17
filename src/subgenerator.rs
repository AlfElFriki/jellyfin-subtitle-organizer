use std::path::{Path, PathBuf};

pub fn generate_subtitle_path(
	video_file: &Path,
	lang: &str,
	index: usize,
	extension: &str,
) -> PathBuf {
	let video_dir = video_file.parent().unwrap();
	let stem = video_file.file_stem().unwrap().to_str().unwrap();

	// Jellyfin format: VideoName.lang.srt (or VideoName.lang.1.srt for multiples)
	let new_filename = if index == 1 {
		format!("{}.{}.{}", stem, lang, extension)
	} else {
		format!("{}.{}.{}.{}", stem, lang, index, extension)
	};

	video_dir.join(new_filename)
}

pub fn move_or_link_subtitle(subtitle: &PathBuf, new_path: &PathBuf) {
	// Try to create a symlink first, fall back to copy if that fails
	println!(
		"Linking: {:?} -> {:?}",
		subtitle.file_name().unwrap(),
		new_path.file_name().unwrap()
	);

	if new_path.exists() {
		println!("  Skipping: destination already exists");
		return;
	}

	match std::os::unix::fs::symlink(subtitle, new_path) {
		Ok(_) => println!("  Created symlink"),
		Err(e) => {
			println!("  Symlink failed ({}), trying copy...", e);
			match std::fs::copy(subtitle, new_path) {
				Ok(_) => println!("  Copied successfully"),
				Err(e) => eprintln!("  Failed to copy: {}", e),
			}
		}
	}
}
