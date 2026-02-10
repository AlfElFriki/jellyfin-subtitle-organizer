/*
* The main use for this program is to recognize subtitles inside a show's folder and move (or link)
* them to the correct location for Jellyfin to pick them up.
* This means that the program needs to:
* 1. Scan a given directory for video files and subtitle files.
* 2. Match subtitle files to their corresponding video files based on naming conventions.
* 3. Move or create symbolic links for the subtitle files to the same directory as their
*    corresponding video files (depending on user flag).
*
* Jellyfin expects subtitle files to be named just like the video files, with the addition of
* language and format indicators. For example:
* - Video file: Show.S01E01.mkv
* - Subtitle file: Show.S01E01.en.srt
* (If more than one subtitle is found for a video it will add a numeric suffix:
* Show.S01E01.en.1.srt, Show.S01E01.en.2.srt, etc.)
*
* The program should also handle edge cases, such as:
* - Multiple subtitle files for the same video file.
* - Subtitle files that do not match any video file.
* - Subtitle files inside a nested directory.
*
*/

use std::collections::HashMap;
use std::path::PathBuf;

fn main() {
	// let show_directory = PathBuf::from("/home/alfelfriki/Downloads/The Walking Dead");

	// For now lets assume cwd is the show directory
	let show_directory = std::env::current_dir().unwrap();

	// Scan the directory for video and subtitle files
	let (video_files, subtitle_files) = scan_directory(&show_directory);

	// Match subtitle files to their corresponding video files
	let matched_subtitles = match_subtitles(&video_files, &subtitle_files);

	// Move or link subtitle files to the correct location
	for (video_file, subtitles) in matched_subtitles {
		// Group subtitles by language to handle multiple subs of the same language
		let mut lang_counts: HashMap<String, usize> = HashMap::new();

		for subtitle in subtitles {
			let lang = extract_language(&subtitle, &video_file);
			let count = lang_counts.entry(lang.clone()).or_insert(0);
			*count += 1;

			let extension = subtitle.extension().unwrap().to_str().unwrap();
			let new_subtitle_path = generate_subtitle_path(&video_file, &lang, *count, extension);
			move_or_link_subtitle(&subtitle, &new_subtitle_path);

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

	// println!("Video files found: {:?}", video_files);
	// println!("Subtitle files found: {:?}", subtitle_files);
	// println!("Matched subtitles: {:?}", matched_subtitles);
}

// Do the actual functions, no placeholders

fn scan_directory(dir: &PathBuf) -> (Vec<PathBuf>, Vec<PathBuf>) {
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

fn match_subtitles(
	video_files: &Vec<PathBuf>,
	subtitle_files: &Vec<PathBuf>,
) -> HashMap<PathBuf, Vec<PathBuf>> {
	let mut matched_subtitles: HashMap<PathBuf, Vec<PathBuf>> = HashMap::new();

	for video in video_files {
		let video_stem = video.file_stem().unwrap().to_str().unwrap();

		for subtitle in subtitle_files {
			let subtitle_file_name = subtitle.file_name().unwrap().to_str().unwrap();
			// Check if subtitle filename starts with video filename stem
			if subtitle_file_name.starts_with(video_stem) {
				matched_subtitles
					.entry(video.clone())
					.or_insert_with(Vec::new)
					.push(subtitle.clone());
			}
		}
	}

	matched_subtitles
}

/// Extract language from subtitle filename by comparing with video filename
/// For example: "Movie.Name.English.srt" compared to "Movie.Name.mkv" -> "English"
/// Returns an ISO 639-1 language code for Jellyfin compatibility
fn extract_language(subtitle: &PathBuf, video: &PathBuf) -> String {
	let video_stem = video.file_stem().unwrap().to_str().unwrap();
	let subtitle_stem = subtitle.file_stem().unwrap().to_str().unwrap();

	// The language is what comes after the video stem in the subtitle filename
	// e.g., "Movie.Name.English" - "Movie.Name" = ".English"
	let lang_part = if subtitle_stem.len() > video_stem.len() {
		let suffix = &subtitle_stem[video_stem.len()..];
		// Remove leading dot if present
		suffix.trim_start_matches('.')
	} else {
		"und" // undefined
	};

	// Convert language names to ISO 639-1 codes for Jellyfin
	language_to_iso639(lang_part)
}

/// Convert language name to ISO 639-1 code
fn language_to_iso639(lang: &str) -> String {
	match lang.to_lowercase().as_str() {
		"arabic" => "ara".to_string(),
		"bulgarian" => "bul".to_string(),
		"chinese" => "chi".to_string(),
		"croatian" => "hrv".to_string(),
		"czech" => "ces".to_string(),
		"danish" => "dan".to_string(),
		"dutch" => "dut".to_string(),
		"english" => "eng".to_string(),
		"estonian" => "est".to_string(),
		"french" => "fre".to_string(),
		"german" => "ger".to_string(),
		"greek" => "gre".to_string(),
		"hebrew" => "heb".to_string(),
		"hungarian" => "hun".to_string(),
		"icelandic" => "ice".to_string(),
		"indonesian" => "ind".to_string(),
		"italian" => "ita".to_string(),
		"japanese" => "jpn".to_string(),
		"korean" => "kor".to_string(),
		"latvian" => "lav".to_string(),
		"lithuanian" => "lit".to_string(),
		"norwegian" => "nor".to_string(),
		"polish" => "pol".to_string(),
		"portuguese" => "por".to_string(),
		"romanian" => "rum".to_string(),
		"russian" => "rus".to_string(),
		"serbian" => "srp".to_string(),
		"slovak" => "slo".to_string(),
		"slovenian" => "slv".to_string(),
		"spanish" => "spa".to_string(),
		"swedish" => "swe".to_string(),
		"thai" => "tha".to_string(),
		"turkish" => "tur".to_string(),
		"ukrainian" => "ukr".to_string(),
		"vietnamese" => "vie".to_string(),
		// If already a 2-3 letter code, use it as-is
		code if code.len() <= 3 => code.to_lowercase(),
		// Fallback to undefined
		_ => "und".to_string(),
	}
}

fn generate_subtitle_path(
	video_file: &PathBuf,
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

fn move_or_link_subtitle(subtitle: &PathBuf, new_path: &PathBuf) {
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
