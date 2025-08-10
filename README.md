# sb — Terminal Markdown Browser/Editor

`sb` (Saorsa Browser) is a fast, secure, keyboard-centric terminal UI for browsing and editing Markdown notes and code. It blends a rendered Markdown preview, inline media, Git-aware diffs, and inline editing into a single, ergonomic interface with enterprise-grade security features.

## Highlights

### Core Features
- **Two-pane UI**: Left file tree (toggleable) + right unified preview/editor
- **Rendered Markdown Preview**: Inline raw-line overlay for current cursor line (high-contrast, line-numbered gutter)
- **Media Support**: Inline images and videos via `[video](path)` links with ffmpeg playback
- **Syntax Highlighting**: Full support for code (Rust, TypeScript, JavaScript, Python) via syntect
- **Git Integration**: Real-time diff vs HEAD with visual change indicators
- **Inline Editing**: Per-line editing in preview mode or full editor pane
- **Command Mode**: Vim-style `:w`, `:q`, `:wq` commands
- **File Operations**: Midnight-Commander-style Copy/Move/Mkdir/Delete

### Security & Performance Features
- **🔒 Security Hardening**: Complete path traversal protection and input validation
- **⚡ Async I/O**: High-performance file operations with Tokio
- **💾 Intelligent Caching**: Directory and file preview caching with TTL
- **📊 Performance Monitoring**: Built-in metrics tracking and benchmarking
- **🔍 Structured Logging**: Comprehensive audit trail with security event tracking
- **🛡️ Resource Protection**: File size limits and memory-bounded operations

## Security & Architecture

### Security Features
- **Path Traversal Protection**: Comprehensive validation prevents access outside allowed directories
- **File Size Limits**: Configurable limits prevent resource exhaustion (default: 10MB max file, 1MB preview)
- **Input Sanitization**: All user inputs validated and sanitized for security
- **Error Message Sanitization**: Sensitive information stripped from error messages
- **Audit Logging**: Security events logged with structured data for monitoring

### Performance Architecture
- **Async I/O**: Non-blocking file operations using Tokio for responsive UI
- **Multi-layer Caching**: 
  - Directory cache with 30s TTL
  - File preview cache with 60s TTL  
  - LRU eviction with memory bounds
- **Streaming**: Large files handled via streaming to prevent memory issues
- **Metrics**: Built-in performance monitoring with timing and cache hit rates

### Testing & Quality
- **80%+ Test Coverage**: Comprehensive unit, integration, and security tests
- **Property-based Testing**: Edge case validation with randomized inputs
- **Security Test Suite**: Dedicated tests for vulnerability prevention
- **Performance Benchmarks**: Automated performance regression detection

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

# Run with debug logging
RUST_LOG=debug ./target/release/sb /path/to/your/notes

# Run performance demo
cargo run --example performance_demo

# Run benchmarks
cargo bench
```

## Configuration

The application supports environment-based configuration:

- `RUST_LOG`: Logging level (error, warn, info, debug, trace)
- Security settings are configured at compile time for maximum safety
- Default limits: 10MB max file size, 1MB preview size
- Cache settings: 30s directory TTL, 60s file preview TTL

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
  - Scroll: Up/Down arrows or j/k; mouse wheel supported
  - Raw edit toggle: press `e` to enter full raw edit; Esc to return to preview
- Editor (Right pane)
  - `:` to open command prompt; supported: `:w`, `:q`, `:wq`
  - Scroll with mouse wheel
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

### General Issues
- **Preview shows raw text, not rendered Markdown**
  - Ensure you opened a `.md` file. The preview renders Markdown; only the current line overlays as raw (yellow) for edit context.
- **ffmpeg playback doesn't work**
  - Verify `ffmpeg` is installed and in PATH.
- **Terminal rendering issues**
  - Use a modern terminal emulator; ensure truecolor support.
- **Git diff not shown**
  - File must be tracked. Ensure repo is initialized and the file has a HEAD version.

### Security & Performance
- **File access denied**
  - Path validation active - files must be within the specified base directory
  - Check permissions and ensure path doesn't contain `../` or hidden files
- **File too large error**
  - Default limit is 10MB for files, 1MB for previews
  - Large files are processed via streaming to prevent memory issues
- **Slow performance**
  - Enable debug logging with `RUST_LOG=debug` to see cache hit rates
  - Directory caching improves repeated navigation performance
  - Check available memory if handling many large files

### Monitoring & Debugging
- **Enable structured logging**: Set `RUST_LOG=debug` for detailed performance metrics
- **View security events**: Security validation failures are logged with context
- **Performance metrics**: Cache hit rates and operation timings included in debug logs

## Roadmap

- Richer motions/operations in Preview and Editor (Home/End/words, undo/redo)
- Inline code block rendering improvements and scrolling
- Video timing and seeking (not only last-frame display)
- Configurable theme and keybindings

## Notes

- This is a terminal UI; mouse is optional and limited.
- Tested on macOS. Linux should work with a recent Rust and `ffmpeg`.
