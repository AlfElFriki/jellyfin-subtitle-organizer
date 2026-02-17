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

mod language_parser;
mod subgenerator;
mod subgetter;

use language_parser::*;
use subgenerator::*;
use subgetter::*;

use std::collections::HashMap;

fn main() {
	let args = std::env::args().skip(1).collect::<Vec<_>>();

	//let show_directory = PathBuf::from("/home/alfelfriki/Downloads/The Big Lebowski");

	// For now let's assume cwd is the show directory
	let show_directory = std::env::current_dir().unwrap();

	// Scan the directory for video and subtitle files
	let (video_files, subtitle_files) = scan_directory(&show_directory);

	// Match subtitle files to their corresponding video files
	let matched_subtitles = match_subtitles(&video_files, &subtitle_files);

	// Move or link subtitle files to the correct location
	for (video_file, subtitles) in &matched_subtitles {
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

	if args.contains(&"DEBUG".to_string()) {
		println!("Video files found: {:?}", video_files);
		println!("Subtitle files found: {:?}", subtitle_files);
		println!("Matched subtitles: {:?}", matched_subtitles);
	}
}
