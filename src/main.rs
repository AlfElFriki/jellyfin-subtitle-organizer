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
mod subdedupper;
mod subgenerator;
mod subgetter;

use subdedupper::*;
use subgenerator::*;
use subgetter::*;

fn main() {
	let args = std::env::args()
		.skip(1)
		.map(|x| x.to_lowercase().trim().to_string())
		.collect::<Vec<_>>();
	dbg!("Arguments: {:?}", &args);

	//let show_directory = PathBuf::from("/home/alfelfriki/Downloads/The Walking Dead");

	let show_directory = std::env::current_dir().unwrap();

	let (video_files, mut subtitle_files) = scan_directory(&show_directory);

	if args.contains(&"debug".to_string()) {
		println!("Video files found: {:?}", &video_files);
		println!("Subtitle files found: {:?}", &subtitle_files);
	}

	if args.contains(&"rename".to_string()) {
		rename_subtitles(&subtitle_files)
			.unwrap_or_else(|e| eprintln!("Error renaming subtitles: {}", e));
		(_, subtitle_files) = scan_directory(&show_directory);
	}

	let matched_subtitles =
		if args.contains(&"generate".to_string()) || args.contains(&"dedup".to_string()) {
			Some(match_subtitles(&video_files, &subtitle_files))
		} else {
			None
		};

	if let Some(matched_subtitles) = &matched_subtitles
		&& args.contains(&"generate".to_string())
	{
		if args.contains(&"debug".to_string()) {
			println!("Matched subtitles: {:?}", &matched_subtitles);
		}

		generate_paths(&matched_subtitles);
	}

	if args.contains(&"dedup".to_string()) {
		dedup_subtitles(matched_subtitles.as_ref().unwrap())
			.unwrap_or_else(|e| eprintln!("Error deduping subtitles: {}", e));
	}
}
