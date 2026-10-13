use std::path::Path;

/// Extract language from subtitle filename by comparing with video filename
/// For example: "Movie.Name.English.srt" compared to "Movie.Name.mkv" -> "English"
/// Returns an ISO 639-1 language code for Jellyfin compatibility
pub fn extract_language(subtitle: &Path, video: &Path) -> String {
	let video_stem = video
		.file_stem()
		.and_then(|stem| stem.to_str())
		.unwrap_or("");
	let subtitle_stem = subtitle
		.file_stem()
		.and_then(|stem| stem.to_str())
		.unwrap_or("");

	let lang_part = subtitle_stem
		.strip_prefix(video_stem)
		.and_then(|suffix| suffix.strip_prefix('.'))
		.unwrap_or(subtitle_stem);
	let tokens = lang_part
		.split(|character: char| !character.is_ascii_alphanumeric())
		.filter(|token| !token.is_empty())
		.collect::<Vec<_>>();
	let language = tokens.first().copied().unwrap_or("und");
	let is_sdh = tokens
		.iter()
		.skip(1)
		.any(|token| token.eq_ignore_ascii_case("sdh"));

	// Convert language names to ISO 639-1 codes for Jellyfin
	let language = language_to_iso639(language);
	if is_sdh {
		format!("{}.sdh", language)
	} else {
		language
	}
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

#[cfg(test)]
mod tests {
	use super::extract_language;
	use std::path::Path;

	#[test]
	fn extracts_language_from_language_named_subs_files() {
		let video = Path::new("The.Hunger.Games.mkv");

		assert_eq!(
			extract_language(Path::new("English(SDH).srt"), video),
			"eng.sdh"
		);
		assert_eq!(extract_language(Path::new("Spanish.srt"), video), "spa");
	}

	#[test]
	fn extracts_language_and_sdh_from_jellyfin_filenames() {
		assert_eq!(
			extract_language(
				Path::new("The.Hunger.Games.eng.0.srt"),
				Path::new("The.Hunger.Games.mkv")
			),
			"eng"
		);
		assert_eq!(
			extract_language(
				Path::new("The.Hunger.Games.eng.sdh.srt"),
				Path::new("The.Hunger.Games.mkv")
			),
			"eng.sdh"
		);
		assert_eq!(
			extract_language(
				Path::new("English.sdh.srt"),
				Path::new("The.Hunger.Games.mkv")
			),
			"eng.sdh"
		);
	}
}
