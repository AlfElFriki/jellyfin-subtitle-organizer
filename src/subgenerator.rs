use crate::language_parser::extract_language;
use std::collections::HashMap;
use std::path::{Path, PathBuf};

pub fn generate_paths(matched_subtitles: &HashMap<PathBuf, Vec<PathBuf>>) {
	// Move or link subtitle files to the correct location
	for (video_file, subtitles) in matched_subtitles {
		// Group subtitles by language to handle multiple subs of the same language
		let mut lang_counts: HashMap<String, usize> = HashMap::new();

		for subtitle in subtitles {
			let lang = extract_language(subtitle, video_file);
			let count = lang_counts.entry(lang.clone()).or_insert(0);
			*count += 1;

			let extension = subtitle.extension().unwrap().to_str().unwrap();
			let new_subtitle_path = generate_subtitle_path(video_file, &lang, *count, extension);
			move_or_link_subtitle(subtitle, &new_subtitle_path);

			// For .idx files, also handle the corresponding .sub file
			if extension == "idx" {
				let sub_file = subtitle.with_extension("sub");
				if sub_file.exists() {
					let new_sub_path = new_subtitle_path.with_extension("sub");
					move_or_link_subtitle(&sub_file, &new_sub_path);
				}
			}
		}
	}
}

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
