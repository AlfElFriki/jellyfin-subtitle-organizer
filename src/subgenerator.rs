use crate::language_parser::extract_language;
use std::collections::{BTreeSet, HashMap};
use std::path::{Path, PathBuf};

pub fn generate_paths(
	matched_subtitles: &HashMap<PathBuf, Vec<PathBuf>>,
	use_symlinks: bool,
	delete_source: bool,
) -> Result<(), String> {
	let mut source_dirs = BTreeSet::new();

	for (video_file, subtitles) in matched_subtitles {
		let mut lang_counts: HashMap<String, usize> = HashMap::new();

		for subtitle in subtitles {
			let lang = extract_language(subtitle, video_file);
			let count = lang_counts.entry(lang.clone()).or_insert(0);
			*count += 1;

			let extension = subtitle
				.extension()
				.and_then(|ext| ext.to_str())
				.ok_or_else(|| format!("Missing or invalid extension: {}", subtitle.display()))?;
			let new_subtitle_path = generate_subtitle_path(video_file, &lang, *count, extension);
			let moved = move_or_link_subtitle(subtitle, &new_subtitle_path, use_symlinks)?;

			if delete_source
				&& subtitle
					.parent()
					.and_then(Path::file_name)
					.is_some_and(|name| name.eq_ignore_ascii_case("Subs"))
			{
				if let Some(source_dir) = subtitle.parent() {
					source_dirs.insert(source_dir.to_path_buf());
				}
			}

			if extension == "idx" && moved {
				let sub_file = subtitle.with_extension("sub");
				if sub_file.exists() {
					let new_sub_path = new_subtitle_path.with_extension("sub");
					move_or_link_subtitle(&sub_file, &new_sub_path, use_symlinks)?;
				}
			}
		}
	}

	for source_dir in source_dirs {
		match std::fs::remove_dir(&source_dir) {
			Ok(()) => println!("Removed empty source directory: {}", source_dir.display()),
			Err(error)
				if error.kind() == std::io::ErrorKind::NotFound
					|| error.kind() == std::io::ErrorKind::DirectoryNotEmpty => {}
			Err(error) => {
				return Err(format!(
					"Failed to remove source directory {}: {}",
					source_dir.display(),
					error
				));
			}
		}
	}

	Ok(())
}

pub fn generate_subtitle_path(
	video_file: &Path,
	lang: &str,
	index: usize,
	extension: &str,
) -> PathBuf {
	let video_dir = video_file.parent().unwrap();
	let stem = video_file.file_stem().unwrap().to_str().unwrap();

	let new_filename = if index == 1 {
		format!("{}.{}.{}", stem, lang, extension)
	} else {
		format!("{}.{}.{}.{}", stem, lang, index, extension)
	};

	video_dir.join(new_filename)
}

fn move_or_link_subtitle(
	subtitle: &Path,
	new_path: &Path,
	use_symlink: bool,
) -> Result<bool, String> {
	println!(
		"{}: {:?} -> {:?}",
		if use_symlink { "Linking" } else { "Moving" },
		subtitle.file_name().unwrap_or_default(),
		new_path.file_name().unwrap_or_default()
	);

	if new_path.exists() {
		println!("  Skipping: destination already exists");
		return Ok(false);
	}

	if use_symlink {
		return std::os::unix::fs::symlink(subtitle, new_path)
			.map(|()| {
				println!("  Created symlink");
				true
			})
			.map_err(|error| {
				format!(
					"Failed to symlink {} to {}: {}",
					subtitle.display(),
					new_path.display(),
					error
				)
			});
	}

	match std::fs::rename(subtitle, new_path) {
		Ok(()) => {
			println!("  Moved successfully");
			Ok(true)
		}
		Err(rename_error) => {
			std::fs::copy(subtitle, new_path).map_err(|copy_error| {
				format!(
					"Failed to move {} to {} (rename: {}; copy: {})",
					subtitle.display(),
					new_path.display(),
					rename_error,
					copy_error
				)
			})?;
			std::fs::remove_file(subtitle).map_err(|error| {
				format!(
					"Copied {} to {}, but failed to remove the source: {}",
					subtitle.display(),
					new_path.display(),
					error
				)
			})?;
			println!("  Moved successfully");
			Ok(true)
		}
	}
}

#[cfg(test)]
mod tests {
	use super::generate_paths;
	use std::collections::HashMap;
	use std::path::PathBuf;

	fn temp_dir() -> PathBuf {
		let path = std::env::temp_dir().join(format!(
			"jellysub-generate-{}-{}",
			std::process::id(),
			std::thread::current().name().unwrap_or("test")
		));
		std::fs::create_dir_all(&path).unwrap();
		path
	}

	#[test]
	fn moves_subtitles_by_default_and_removes_empty_subs_directory_on_request() {
		let directory = temp_dir();
		let source_dir = directory.join("Subs");
		std::fs::create_dir(&source_dir).unwrap();
		let video = directory.join("Movie.mkv");
		let subtitle = source_dir.join("Movie.eng.srt");
		std::fs::write(&video, b"video").unwrap();
		std::fs::write(&subtitle, b"subtitle").unwrap();
		let matches = HashMap::from([(video, vec![subtitle.clone()])]);

		generate_paths(&matches, false, true).unwrap();

		assert!(!subtitle.exists());
		assert_eq!(
			std::fs::read(directory.join("Movie.eng.srt")).unwrap(),
			b"subtitle"
		);
		assert!(!source_dir.exists());
		std::fs::remove_dir_all(directory).unwrap();
	}

	#[test]
	fn retains_nonempty_subs_directory() {
		let directory = temp_dir();
		let source_dir = directory.join("Subs");
		std::fs::create_dir(&source_dir).unwrap();
		let video = directory.join("Movie.mkv");
		let subtitle = source_dir.join("Movie.eng.srt");
		let leftover = source_dir.join("notes.txt");
		std::fs::write(&video, b"video").unwrap();
		std::fs::write(&subtitle, b"subtitle").unwrap();
		std::fs::write(&leftover, b"keep").unwrap();
		let matches = HashMap::from([(video, vec![subtitle])]);

		generate_paths(&matches, false, true).unwrap();

		assert!(source_dir.exists());
		assert_eq!(std::fs::read(leftover).unwrap(), b"keep");
		std::fs::remove_dir_all(directory).unwrap();
	}

	#[test]
	fn symlink_mode_keeps_source_subtitle() {
		let directory = temp_dir();
		let source_dir = directory.join("Subs");
		std::fs::create_dir(&source_dir).unwrap();
		let video = directory.join("Movie.mkv");
		let subtitle = source_dir.join("Movie.eng.srt");
		let target = directory.join("Movie.eng.srt");
		std::fs::write(&video, b"video").unwrap();
		std::fs::write(&subtitle, b"subtitle").unwrap();
		let matches = HashMap::from([(video, vec![subtitle.clone()])]);

		generate_paths(&matches, true, false).unwrap();

		assert!(subtitle.exists());
		assert!(
			std::fs::symlink_metadata(target)
				.unwrap()
				.file_type()
				.is_symlink()
		);
		std::fs::remove_dir_all(directory).unwrap();
	}

	#[test]
	fn generates_from_language_named_subs_directory() {
		let directory = temp_dir();
		let source_dir = directory.join("Subs");
		std::fs::create_dir(&source_dir).unwrap();
		let video = directory.join("The.Hunger.Games.mkv");
		std::fs::write(&video, b"video").unwrap();
		std::fs::write(source_dir.join("English(SDH).srt"), b"english sdh").unwrap();
		std::fs::write(source_dir.join("English.srt"), b"english").unwrap();
		std::fs::write(source_dir.join("Spanish.srt"), b"spanish").unwrap();

		let (videos, subtitles) = crate::subgetter::scan_directory(&directory);
		let matches = crate::subgetter::match_subtitles(&videos, &subtitles);
		assert_eq!(matches.get(&video).unwrap().len(), 3);

		generate_paths(&matches, false, true).unwrap();

		assert_eq!(
			std::fs::read(directory.join("The.Hunger.Games.eng.srt")).unwrap(),
			b"english"
		);
		assert_eq!(
			std::fs::read(directory.join("The.Hunger.Games.eng.sdh.srt")).unwrap(),
			b"english sdh"
		);
		assert_eq!(
			std::fs::read(directory.join("The.Hunger.Games.spa.srt")).unwrap(),
			b"spanish"
		);
		assert!(!source_dir.exists());
		std::fs::remove_dir_all(directory).unwrap();
	}
}
