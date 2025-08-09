# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

`sb` (Saorsa Browser) is a terminal-based Markdown browser/editor with Vim-like features, built in Rust using the Ratatui TUI framework. It provides a dual-pane interface with file navigation, rendered Markdown preview, inline editing, syntax highlighting, Git diffs, and media support (images/videos via ffmpeg).

## Architecture

### Core Components

- **main.rs**: Application entry point, event loop, and keyboard handling
- **app.rs**: Application state management, file operations, and command processing
  - Manages focus states (Left pane, Preview, Editor)
  - Handles file operations (Copy/Move/Mkdir/Delete)
  - Video playback via ffmpeg subprocess
  - Vim mode implementation
- **preview.rs**: Markdown rendering, syntax highlighting, and media handling
  - Uses `tui-markdown` for Markdown to Ratatui Text conversion
  - `syntect` for syntax highlighting (rs, ts/tsx, js/jsx, py)
  - Git diff generation via `git show HEAD:<file>`
  - Image rendering with `ratatui-image`
- **fs.rs**: File system utilities and tree building

### Key Dependencies

- **TUI**: `ratatui` (0.29) + `crossterm` for terminal UI
- **Markdown**: `tui-markdown` for rendering
- **Editing**: `tui-textarea` for text input
- **File tree**: `tui-tree-widget` for navigation
- **Syntax**: `syntect` for code highlighting
- **Media**: `ratatui-image` for images, ffmpeg subprocess for video

## Development Commands

```bash
# Build
cargo build --release

# Run with a directory
cargo run --release -- /path/to/notes

# Run with current directory
cargo run --release

# Format code
cargo fmt

# Lint code
cargo clippy

# Run tests (if any exist)
cargo test

# Check compilation without building
cargo check

# Build optimized binary
cargo build --release --target=x86_64-apple-darwin  # for macOS
```

## Code Patterns

### Event Handling
The main event loop in `main.rs` uses a state-based approach:
- Different key handlers for different modes (file creation, picker, command mode, etc.)
- Focus determines which pane receives input
- Modal overlays (help, delete confirmation, file picker) take precedence

### Vim Mode Implementation
Located in `app.rs`:
- `VimMode` enum: Normal/Insert
- Commands: `j/k` (movement), `dd` (delete), `yy/p` (yank/paste)
- Inline editing: `i/e` enters edit mode for current line
- Command mode: `:w`, `:q`, `:wq`, `:v` (toggle vim)

### File Operations
MC-style file operations in `app.rs`:
- F5: Copy, F6: Move, F7: Mkdir, F8: Delete
- Uses picker overlay for destination selection
- Immediate save on inline edits

### Preview Rendering
The preview system (`preview.rs`) layers multiple elements:
- Base: Rendered Markdown text
- Overlay: Current line in raw format with line numbers
- Media: Images inline, video playback area
- Code blocks: Syntax highlighted with optional Git diff

## Prerequisites

- **ffmpeg**: Required in PATH for video playback
  - macOS: `brew install ffmpeg`
  - Linux: `sudo apt-get install ffmpeg`
- **Git**: Required for diff functionality
- **Rust**: Stable toolchain

## Testing Approach

Currently no automated tests. Manual testing workflow:
1. Build with `cargo build --release`
2. Test file navigation with sample Markdown directory
3. Verify preview rendering, inline editing
4. Test media display (images/videos)
5. Verify Git diff for tracked files
6. Test all file operations (Copy/Move/Delete/Mkdir)

## Common Issues

- **Video playback fails**: Ensure ffmpeg is installed and in PATH
- **Git diff not showing**: File must be tracked in Git with HEAD version
- **Terminal rendering issues**: Requires modern terminal with truecolor support
- **Syntax highlighting**: Only supports rs, ts/tsx, js/jsx, py files

## Performance Considerations

- Tree building is synchronous (may lag on large directories)
- Video playback spawns ffmpeg subprocess
- Syntax highlighting cached via `once_cell::Lazy`
- Release build uses aggressive optimizations (see Cargo.toml profile)