# Jellyfin Subtitle Organizer

Run `jellysub` from a media directory. It scans for `.mkv`, `.mp4`, and `.avi` videos and `.srt`, `.ass`, and `.idx` subtitles. Language-named files in a sibling `Subs` folder (such as `English.srt`) are matched when there is only one video in that directory.

```text
jellysub generate [--symlink] [--delete-source]
jellysub dedup
jellysub rename
```

- `generate` names subtitles for Jellyfin and moves them next to their video by default. `--symlink` creates links instead. `--delete-source` removes empty `Subs` folders after successful moves; it cannot be used with `--symlink`.
- `dedup` removes identical subtitles belonging to the same video version, preferring an unsuffixed filename when available.
- `rename` removes numeric suffixes such as `.0` only when the unsuffixed filename does not already exist.

Add `--debug` to print discovered files and matches. Use `jellysub --help` for help.
