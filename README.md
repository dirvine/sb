uio# sb — Terminal Markdown Browser/Editor

`sb` is a fast, keyboard-centric terminal UI for browsing and editing Markdown notes and code. It blends a rendered Markdown preview, inline media, Git-aware diffs, and inline editing into a single, ergonomic interface.

## Highlights

- Two-pane UI
  - Left: file tree (toggleable)
  - Right: unified preview/editor
- Rendered Markdown preview with an inline raw-line overlay for the current cursor line (high-contrast, line-numbered gutter)
- Images shown inline; videos discovered from `[video](path)` links
  - ffmpeg CLI playback in the preview (autoplays the first video link); Space to pause/resume, `s` to stop
- Syntax highlighting for code (rs, ts/tsx, js/jsx, py) via syntect
- Git-aware diff vs HEAD (split below highlighted code when changes exist)
- Inline per-line editing in the preview (press `e` or `i`, Enter to save immediately)
- Editor pane `:` command mode
  - `:w`, `:q`, `:wq`
- Midnight-Commander-style file ops: Copy/Move/Mkdir/Delete

## Prerequisites

- Rust toolchain (stable)
- ffmpeg in your PATH (required for video playback)
  - macOS: `brew install ffmpeg`
  - Ubuntu/Debian: `sudo apt-get install ffmpeg`

## Build & Run

```bash
# Build
cargo build --release

# Run (open a directory of notes)
cargo run --release -- /path/to/your/notes

# Or run the compiled binary
./target/release/sb /path/to/your/notes
```

## Keybindings

- Pane focus
  - Tab / Shift+Tab: cycle focus (Left ↔ Preview ↔ Editor).
  - Ctrl+B or F9: toggle Files pane.
- Files (Left pane)
  - Enter: toggle folder / open file
  - F5 Copy, F6 Move, F7 Mkdir, F8 Delete
  - N: new file (Markdown suggested)
- Preview (Right pane)
  - Rendered Markdown with raw-line overlay (yellow) + line number gutter
  - Inline edit: `e` or `i` to edit current line; Enter saves immediately; Esc cancels
- Editor (Right pane)
  - `:` to open command prompt; supported: `:w`, `:q`, `:wq`
- Video playback (from `[video](path)` links in Markdown)
  - Autoplays first link in preview
  - Space: pause/resume, `s`: stop
- General
  - `?` or `h`: help
  - `Q` / Esc: quit (Esc also cancels dialogs)

## How it Works

- Markdown preview
  - Parsed to styled `ratatui::widgets::Paragraph` text.
  - When editing context is needed, the current line overlays on top of the preview (bold black-on-yellow) with a blue line-number gutter, while the rest remains dim-rendered for context.
- Images
  - Loaded via `image` and rendered with `ratatui-image`.
- Videos
  - `ffmpeg` is spawned to output MJPEG frames (`image2pipe`); frames are decoded and displayed in a small area of the preview.
- Code highlighting + diff
  - syntect provides syntax highlighting (rs/ts/tsx/js/jsx/py).
  - `git show HEAD:<relative-path>` is diffed against the current buffer (lines) and shown beneath the code when changes exist.
- Editing
  - Per-line inline edits (Preview) save immediately on Enter.
  - Editor pane offers a minimal `:` command mode.

## Troubleshooting

- Preview shows raw text, not rendered Markdown
  - Ensure you opened a `.md` file. The preview renders Markdown; only the current line overlays as raw (yellow) for edit context.
- ffmpeg playback doesn’t work
  - Verify `ffmpeg` is installed and in PATH.
- Terminal rendering issues
  - Use a modern terminal emulator; ensure truecolor support.
- Git diff not shown
  - File must be tracked. Ensure repo is initialized and the file has a HEAD version.

## Roadmap

- Richer motions/operations in Preview and Editor (Home/End/words, undo/redo)
- Inline code block rendering improvements and scrolling
- Video timing and seeking (not only last-frame display)
- Configurable theme and keybindings

## Notes

- This is a terminal UI; mouse is optional and limited.
- Tested on macOS. Linux should work with a recent Rust and `ffmpeg`.
