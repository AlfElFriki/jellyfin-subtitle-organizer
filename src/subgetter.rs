use std::collections::HashMap;
use std::path::PathBuf;

pub fn scan_directory(dir: &PathBuf) -> (Vec<PathBuf>, Vec<PathBuf>) {
	// Implementation to scan the directory and return video and subtitle files
	let mut video_files = Vec::new();
	let mut subtitle_files = Vec::new();

	for entry in walkdir::WalkDir::new(dir) {
		let entry = entry.unwrap();
		let path = entry.path();

		if path.is_file()
			&& let Some(extension) = path.extension()
		{
			match extension.to_str().unwrap().to_lowercase().as_str() {
				"mkv" | "mp4" | "avi" => video_files.push(path.to_path_buf()),
				// Note: .sub files are handled together with their .idx counterpart
				// so we only track .idx files for VobSub subtitles
				"srt" | "ass" | "idx" => subtitle_files.push(path.to_path_buf()),
				_ => (),
			}
		}
	}

	(video_files, subtitle_files)
}

pub fn match_subtitles(
	video_files: &Vec<PathBuf>,
	subtitle_files: &Vec<PathBuf>,
) -> HashMap<PathBuf, Vec<PathBuf>> {
	let mut matched_subtitles: HashMap<PathBuf, Vec<PathBuf>> = HashMap::new();
	let mut videos_by_directory: HashMap<PathBuf, usize> = HashMap::new();

	for video in video_files {
		if let Some(directory) = video.parent() {
			*videos_by_directory
				.entry(directory.to_path_buf())
				.or_default() += 1;
		}
	}

	for video in video_files {
		let video_stem = video.file_stem().unwrap().to_str().unwrap();
		let video_directory = video.parent();

		for subtitle in subtitle_files {
			let subtitle_file_name = subtitle.file_name().unwrap().to_str().unwrap();
			// A subtitle belongs to this video only when the video stem is a
			// complete prefix, followed by Jellyfin's language separator.
			let has_video_prefix = subtitle_file_name.starts_with(&format!("{}.", video_stem));

			// Language-named files in a sibling Subs directory are unambiguous
			// only when that directory contains a single video.
			let is_unique_subs_match = subtitle
				.parent()
				.filter(|directory| {
					directory
						.file_name()
						.is_some_and(|name| name.eq_ignore_ascii_case("Subs"))
				})
				.and_then(|directory| directory.parent())
				.zip(video_directory)
				.is_some_and(|(subs_parent, video_parent)| {
					subs_parent == video_parent && videos_by_directory.get(video_parent) == Some(&1)
				});

			if has_video_prefix || is_unique_subs_match {
				matched_subtitles
					.entry(video.clone())
					.or_default()
					.push(subtitle.clone());
			}
		}
	}

	matched_subtitles
}

#[cfg(test)]
mod tests {
	use super::match_subtitles;
	use std::path::PathBuf;

	#[test]
	fn keeps_versioned_video_subtitles_in_separate_matches() {
		let videos = vec![
			PathBuf::from("Movie - [4K].mkv"),
			PathBuf::from("Movie - [Spanish].avi"),
		];
		let subtitles = vec![
			PathBuf::from("Movie - [4K].eng.srt"),
			PathBuf::from("Movie - [Spanish].spa.srt"),
		];

		let matches = match_subtitles(&videos, &subtitles);

		assert_eq!(matches.get(&videos[0]), Some(&vec![subtitles[0].clone()]));
		assert_eq!(matches.get(&videos[1]), Some(&vec![subtitles[1].clone()]));
	}

	#[test]
	fn matches_language_named_subs_folder_for_single_video() {
		let videos = vec![PathBuf::from("/media/Movie/The.Hunger.Games.mkv")];
		let subtitles = vec![
			PathBuf::from("/media/Movie/Subs/English(SDH).srt"),
			PathBuf::from("/media/Movie/Subs/Spanish.srt"),
		];

		let matches = match_subtitles(&videos, &subtitles);

		assert_eq!(matches.get(&videos[0]), Some(&subtitles));
	}

	#[test]
	fn does_not_guess_which_video_owns_subs_in_a_multi_video_directory() {
		let videos = vec![
			PathBuf::from("/media/Movie/Movie.1080p.mkv"),
			PathBuf::from("/media/Movie/Movie.4K.mkv"),
		];
		let subtitles = vec![PathBuf::from("/media/Movie/Subs/English.srt")];

		assert!(match_subtitles(&videos, &subtitles).is_empty());
	}
}
