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

use clap::Parser;
use subdedupper::*;
use subgenerator::*;
use subgetter::*;

#[derive(Debug, Parser)]
#[command(version, about = "Organize and deduplicate Jellyfin subtitles")]
struct Cli {
	/// Generate Jellyfin-compatible subtitle paths.
	#[arg(long)]
	generate: bool,

	/// Remove byte-identical subtitles within each exact video version.
	#[arg(long)]
	dedup: bool,

	/// Rename numbered subtitles to the unsuffixed name when it is available.
	#[arg(long)]
	rename: bool,

	/// Print discovered files and subtitle matches.
	#[arg(long)]
	debug: bool,
}

fn main() {
	let args = Cli::parse();

	//let show_directory = PathBuf::from("/home/alfelfriki/Downloads/The Walking Dead");

	let show_directory = std::env::current_dir().unwrap();

	let (video_files, mut subtitle_files) = scan_directory(&show_directory);

	if args.debug {
		println!("Video files found: {:?}", &video_files);
		println!("Subtitle files found: {:?}", &subtitle_files);
	}

	if args.rename {
		rename_subtitles(&subtitle_files)
			.unwrap_or_else(|e| eprintln!("Error renaming subtitles: {}", e));
		(_, subtitle_files) = scan_directory(&show_directory);
	}

	let matched_subtitles = if args.generate || args.dedup {
		Some(match_subtitles(&video_files, &subtitle_files))
	} else {
		None
	};

	if let Some(matched_subtitles) = &matched_subtitles
		&& args.generate
	{
		if args.debug {
			println!("Matched subtitles: {:?}", &matched_subtitles);
		}

		generate_paths(&matched_subtitles);
	}

	if args.dedup {
		dedup_subtitles(matched_subtitles.as_ref().unwrap())
			.unwrap_or_else(|e| eprintln!("Error deduping subtitles: {}", e));
	}
}

#[cfg(test)]
mod tests {
	use super::Cli;
	use clap::Parser;

	#[test]
	fn parses_combinable_modes_as_flags() {
		let args =
			Cli::try_parse_from(["jellysub", "--generate", "--dedup", "--rename", "--debug"])
				.unwrap();

		assert!(args.generate);
		assert!(args.dedup);
		assert!(args.rename);
		assert!(args.debug);
	}

	#[test]
	fn rejects_unknown_modes() {
		assert!(Cli::try_parse_from(["jellysub", "generate"]).is_err());
	}
}
