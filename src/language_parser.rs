use std::path::Path;

/// Extract language from subtitle filename by comparing with video filename
/// For example: "Movie.Name.English.srt" compared to "Movie.Name.mkv" -> "English"
/// Returns an ISO 639-1 language code for Jellyfin compatibility
pub fn extract_language(subtitle: &Path, video: &Path) -> String {
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
