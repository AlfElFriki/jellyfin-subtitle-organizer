use std::collections::HashMap;
use std::hash::{Hash, Hasher};
use std::path::{Path, PathBuf};

fn find_dupes(subs: &[PathBuf]) -> Vec<PathBuf> {
	// We want to check for subtitles which are EXACTLY the same, so we hash the files and compare that. We return a list of the paths to the duplicate files.

	let mut hashes: HashMap<String, PathBuf> = HashMap::new();
	let mut dupes: Vec<PathBuf> = Vec::new();
	let mut ordered_subs = subs.to_vec();
	ordered_subs.sort_by_key(|path| (has_numeric_suffix(path), path.clone()));

	for sub in ordered_subs {
		let hash = hash_file(&sub);
		if let Some(_existing) = hashes.get(&hash) {
			dupes.push(sub);
		} else {
			hashes.insert(hash, sub.clone());
		}
	}

	dupes
}

fn hash_file(path: &Path) -> String {
	let mut hasher = std::collections::hash_map::DefaultHasher::new();
	std::fs::read(path)
		.unwrap_or_else(|_| panic!("Failed to read file {}", path.display()))
		.hash(&mut hasher);
	hasher.finish().to_string()
}

pub fn dedup_subtitles(matched_subtitles: &HashMap<PathBuf, Vec<PathBuf>>) -> Result<(), String> {
	// Deduplicate only within each video's subtitle set. Different video
	// versions may legitimately contain byte-identical subtitles.
	for subtitles in matched_subtitles.values() {
		for dupe in find_dupes(subtitles) {
			println!("Removing duplicate subtitle: {}", dupe.display());
			std::fs::remove_file(&dupe)
				.map_err(|e| format!("Failed to remove file {}: {}", dupe.display(), e))?;
		}
	}

	Ok(())
}

fn numbered_subtitle_target(path: &Path) -> Option<PathBuf> {
	let stem = path.file_stem()?.to_str()?;
	let (base, suffix) = stem.rsplit_once('.')?;
	if suffix.is_empty() || !suffix.chars().all(|character| character.is_ascii_digit()) {
		return None;
	}

	let mut target = path.to_path_buf();
	target.set_file_name(format!("{}.{}", base, path.extension()?.to_str()?));
	Some(target)
}

fn has_numeric_suffix(path: &Path) -> bool {
	numbered_subtitle_target(path).is_some()
}

pub fn rename_subtitles(subs: &[PathBuf]) -> Result<(), String> {
	for subtitle in subs {
		let Some(target) = numbered_subtitle_target(subtitle) else {
			continue;
		};

		if target.exists() {
			continue;
		}

		println!(
			"Renaming numbered subtitle: {} -> {}",
			subtitle.display(),
			target.display()
		);
		std::fs::rename(subtitle, &target).map_err(|e| {
			format!(
				"Failed to rename {} to {}: {}",
				subtitle.display(),
				target.display(),
				e
			)
		})?;

		// VobSub subtitles use matching .idx and .sub files.
		if subtitle
			.extension()
			.and_then(|extension| extension.to_str())
			== Some("idx")
		{
			let numbered_sub = subtitle.with_extension("sub");
			let target_sub = target.with_extension("sub");
			if numbered_sub.exists() && !target_sub.exists() {
				std::fs::rename(&numbered_sub, &target_sub).map_err(|e| {
					format!(
						"Failed to rename {} to {}: {}",
						numbered_sub.display(),
						target_sub.display(),
						e
					)
				})?;
			}
		}
	}

	Ok(())
}

#[cfg(test)]
mod tests {
	use super::{find_dupes, numbered_subtitle_target};
	use std::path::PathBuf;

	#[test]
	fn prefers_unsuffixed_duplicate() {
		let unsuffixed = PathBuf::from("Movie.spa.srt");
		let numbered = PathBuf::from("Movie.spa.0.srt");

		assert_eq!(
			numbered_subtitle_target(&numbered),
			Some(unsuffixed.clone())
		);
	}

	#[test]
	fn numbered_duplicate_is_removed_when_unsuffixed_is_present() {
		let directory = std::env::temp_dir().join(format!("jellysub-dedup-{}", std::process::id()));
		std::fs::create_dir_all(&directory).unwrap();
		let unsuffixed = directory.join("Movie.spa.srt");
		let numbered = directory.join("Movie.spa.0.srt");
		std::fs::write(&unsuffixed, b"same").unwrap();
		std::fs::write(&numbered, b"same").unwrap();

		let dupes = find_dupes(&[numbered.clone(), unsuffixed.clone()]);

		assert_eq!(dupes, vec![numbered]);
		std::fs::remove_dir_all(directory).unwrap();
	}
}
