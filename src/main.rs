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

use clap::{Parser, Subcommand};
use subdedupper::*;
use subgenerator::*;
use subgetter::*;

#[derive(Debug, Parser)]
#[command(version, about = "Organize and deduplicate Jellyfin subtitles")]
struct Cli {
	/// Print discovered files and subtitle matches.
	#[arg(long, global = true)]
	debug: bool,

	#[command(subcommand)]
	command: Commands,
}

#[derive(Debug, Subcommand)]
enum Commands {
	/// Generate Jellyfin-compatible subtitle paths.
	Generate {
		/// Create symbolic links instead of moving subtitle files.
		#[arg(long, conflicts_with = "delete_source")]
		symlink: bool,

		/// Remove empty `Subs` directories after their subtitles are moved.
		#[arg(long)]
		delete_source: bool,
	},
	/// Remove byte-identical subtitles within each exact video version.
	Dedup,
	/// Rename numbered subtitles to the unsuffixed name when it is available.
	Rename,
}

fn main() {
	let args = Cli::parse();

	let show_directory = std::env::current_dir().unwrap();
	let (video_files, subtitle_files) = scan_directory(&show_directory);

	match args.command {
		Commands::Generate {
			symlink,
			delete_source,
		} => {
			if args.debug {
				println!("Video files found: {:?}", &video_files);
				println!("Subtitle files found: {:?}", &subtitle_files);
			}

			let matched_subtitles = match_subtitles(&video_files, &subtitle_files);
			if args.debug {
				println!("Matched subtitles: {:?}", &matched_subtitles);
			}

			generate_paths(&matched_subtitles, symlink, delete_source)
				.unwrap_or_else(|e| eprintln!("Error generating subtitle paths: {}", e));
		}
		Commands::Dedup => {
			let matched_subtitles = match_subtitles(&video_files, &subtitle_files);
			dedup_subtitles(&matched_subtitles)
				.unwrap_or_else(|e| eprintln!("Error deduping subtitles: {}", e));
		}
		Commands::Rename => rename_subtitles(&subtitle_files)
			.unwrap_or_else(|e| eprintln!("Error renaming subtitles: {}", e)),
	}
}

#[cfg(test)]
mod tests {
	use super::{Cli, Commands};
	use clap::Parser;

	#[test]
	fn parses_generate_options() {
		let args = Cli::try_parse_from(["jellysub", "--debug", "generate", "--symlink"]).unwrap();

		assert!(args.debug);
		assert!(matches!(
			args.command,
			Commands::Generate {
				symlink: true,
				delete_source: false
			}
		));
	}

	#[test]
	fn parses_dedup_and_rename_commands() {
		assert!(matches!(
			Cli::try_parse_from(["jellysub", "dedup"]).unwrap().command,
			Commands::Dedup
		));
		assert!(matches!(
			Cli::try_parse_from(["jellysub", "rename"]).unwrap().command,
			Commands::Rename
		));
		assert!(Cli::try_parse_from(["jellysub", "--generate"]).is_err());
	}

	#[test]
	fn source_removal_cannot_be_combined_with_symlinks() {
		assert!(
			Cli::try_parse_from(["jellysub", "generate", "--symlink", "--delete-source"]).is_err()
		);
	}
}
