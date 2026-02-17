use std::collections::HashMap;
use std::hash::{Hash, Hasher};
use std::path::{Path, PathBuf};

fn find_dupes(subs: &Vec<PathBuf>) -> Vec<PathBuf> {
	// We want to check for subtitles which are EXACTLY the same, so we hash the files and compare that. We return a list of the paths to the duplicate files.

	let mut hashes: HashMap<String, PathBuf> = HashMap::new();
	let mut dupes: Vec<PathBuf> = Vec::new();

	for sub in subs {
		let hash = hash_file(sub);
		if let Some(_existing) = hashes.get(&hash) {
			dupes.push(sub.clone());
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

pub fn dedup_subtitles(subs: &Vec<PathBuf>) -> Result<(), String> {
	let dupes = find_dupes(subs);
	if dupes.is_empty() {
		return Ok(());
	}

	for dupe in dupes {
		println!("Removing duplicate subtitle: {}", dupe.display());
		std::fs::remove_file(&dupe)
			.map_err(|e| format!("Failed to remove file {}: {}", dupe.display(), e))?;
	}

	Ok(())
}
