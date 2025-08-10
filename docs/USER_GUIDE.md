# Saorsa Browser User Guide

## Table of Contents
1. [Getting Started](#getting-started)
2. [Security Features](#security-features)
3. [Performance Features](#performance-features)
4. [Advanced Usage](#advanced-usage)
5. [Troubleshooting](#troubleshooting)
6. [Configuration](#configuration)

---

## Getting Started

### Installation

```bash
# Clone the repository
git clone https://github.com/dirvine/sb
cd sb

# Build and install
cargo build --release

# Run from any directory
./target/release/sb /path/to/your/documents
```

### First Launch

When you first launch Saorsa Browser, you'll see:
- **Left Pane**: File tree of your document directory
- **Right Pane**: Preview/editor area
- **Status Bar**: Current file and system information

### Basic Navigation

- **Tab/Shift+Tab**: Cycle between panes (File Tree ↔ Preview ↔ Editor)
- **Ctrl+B or F9**: Toggle file tree visibility
- **Arrow Keys**: Navigate file tree or scroll preview
- **Enter**: Open file or expand directory
- **Escape**: Return to previous mode or quit

---

## Security Features

### Path Protection

Saorsa Browser implements comprehensive path validation to protect your system:

- **Boundary Enforcement**: Files must be within your specified base directory
- **Traversal Prevention**: Paths like `../../../etc/passwd` are automatically blocked
- **Hidden File Protection**: System files starting with `.` are restricted by default

**What This Means for You:**
- You can browse safely without accidentally accessing system files
- Malicious markdown files cannot trick the browser into opening dangerous locations
- Your file tree is contained within the directory you specify

### File Size Limits

To prevent system overload:
- **Maximum file size**: 10MB (configurable)
- **Preview limit**: 1MB (configurable)
- **Streaming**: Large files are processed in chunks to prevent memory issues

**Practical Impact:**
- Large log files won't freeze the application
- Image files are safely handled with size validation
- Preview generation is fast and memory-efficient

### Error Message Sanitization

Error messages automatically remove sensitive information:
- File paths are generalized (e.g., `/home/user/secret.txt` → `file`)
- IP addresses and credentials are redacted
- System-specific details are masked

---

## Performance Features

### Intelligent Caching

Saorsa Browser uses multi-layer caching for optimal performance:

#### Directory Cache
- **Duration**: 30 seconds (configurable)
- **Benefit**: Instant navigation for recently visited directories
- **Memory Usage**: ~1MB for 1000 directories

#### File Preview Cache
- **Duration**: 60 seconds (configurable)
- **Strategy**: Small files cached, large files streamed
- **Memory Usage**: ~50MB maximum

### Async I/O Operations

All file operations are non-blocking:
- **UI Responsiveness**: Interface never freezes during file operations
- **Background Loading**: Large files load while you continue working
- **Cancellation**: Long operations can be interrupted

### Performance Monitoring

Built-in performance tracking shows:
- Operation timing (directory reads, file previews)
- Cache hit rates (higher is better)
- Memory usage patterns
- System responsiveness metrics

**Viewing Performance Data:**
```bash
# Enable debug logging to see performance metrics
RUST_LOG=debug ./target/release/sb /path/to/docs
```

---

## Advanced Usage

### Configuration via Environment

```bash
# Enable detailed logging
export RUST_LOG=debug

# Launch with logging
./target/release/sb /path/to/docs
```

### Working with Large Files

For files exceeding normal limits:
1. **Streaming Mode**: Large files are automatically streamed
2. **Preview Limits**: First 1MB shown, rest available on demand
3. **Memory Protection**: System memory is protected from exhaustion

### Markdown Features

#### Image Support
```markdown
![Description](path/to/image.png)
```
- Supports PNG, JPEG formats
- Images displayed inline in preview
- Path validation ensures security

#### Video Support
```markdown
[video](path/to/video.mp4)
```
- Requires ffmpeg installation
- Automatic playback in preview
- Space bar to pause/resume, 's' to stop

#### Code Highlighting
```markdown
```rust
fn hello_world() {
    println!("Hello, World!");
}
```
```
- Syntax highlighting for Rust, JavaScript, Python, TypeScript
- Git diff integration shows changes vs HEAD
- Inline editing with immediate save

### File Operations

#### File Tree Operations (F-Key Commands)
- **F5**: Copy file/directory
- **F6**: Move file/directory
- **F7**: Create new directory
- **F8**: Delete file/directory
- **N**: Create new file (defaults to .md extension)

#### Editor Commands
- **:w**: Save file
- **:q**: Quit editor
- **:wq**: Save and quit
- **e** (in preview): Enter edit mode for current line
- **i** (in preview): Insert mode for current line

---

## Troubleshooting

### Common Issues

#### "Path validation failed"
**Cause**: Trying to access files outside the base directory
**Solution**: 
- Ensure files are within your specified document directory
- Check that paths don't contain `../` patterns
- Verify file permissions

#### "File too large" error
**Cause**: File exceeds size limits
**Solution**:
- For viewing: File content will be streamed automatically
- For editing: Consider breaking large files into smaller sections
- Adjust limits if needed (see Configuration section)

#### Slow performance
**Cause**: Cache misses or large file processing
**Solution**:
- Enable debug logging to see cache hit rates: `RUST_LOG=debug`
- Allow time for caches to warm up with usage
- Check available system memory

#### Images not displaying
**Cause**: Unsupported format or file corruption
**Solution**:
- Ensure images are PNG or JPEG format
- Check file integrity
- Verify file permissions

#### Videos not playing
**Cause**: Missing ffmpeg or unsupported format
**Solution**:
```bash
# macOS
brew install ffmpeg

# Ubuntu/Debian
sudo apt-get install ffmpeg

# Verify installation
ffmpeg -version
```

### Debug Mode

Enable comprehensive debugging:

```bash
# Full debug output
RUST_LOG=debug ./target/release/sb /path/to/docs

# Security events only
RUST_LOG=security=debug ./target/release/sb /path/to/docs

# Performance metrics only
RUST_LOG=performance=debug ./target/release/sb /path/to/docs
```

### Performance Analysis

To analyze performance:

1. **Enable debug logging** to see cache hit rates and operation timings
2. **Check system resources** - high memory usage may indicate cache tuning needed
3. **Monitor file sizes** - very large files may need different handling
4. **Review directory structure** - deep nesting can impact performance

---

## Configuration

### Runtime Environment Variables

```bash
# Logging level (error, warn, info, debug, trace)
export RUST_LOG=info

# Performance tuning (planned for future versions)
export SB_CACHE_TTL_SECS=30
export SB_MAX_FILE_SIZE_MB=10
export SB_MAX_PREVIEW_SIZE_MB=1
```

### Compile-time Configuration

Security settings are configured at compile time for maximum safety:
- File size limits defined in `src/security.rs`
- Cache settings in individual cache modules
- Performance thresholds in monitoring code

### Customization

#### Cache Tuning
For different usage patterns:
- **Heavy browsing**: Increase directory cache TTL
- **Large files**: Increase file preview cache size limits
- **Memory constrained**: Reduce cache sizes and TTL

#### Security Tuning
- **Restricted environments**: Enable hidden file blocking
- **Development use**: Adjust file size limits for large assets
- **Multi-user**: Implement additional path restrictions

---

## Best Practices

### For Daily Use

1. **Organize documents** in a dedicated directory structure
2. **Use markdown features** like images and code blocks for rich content
3. **Leverage caching** by revisiting directories to improve performance
4. **Monitor performance** occasionally with debug logging

### For Security

1. **Use dedicated document directories** rather than filesystem root
2. **Be cautious with external markdown** that might reference system files
3. **Keep the application updated** for latest security improvements
4. **Review logs periodically** for unusual security events

### For Performance

1. **Allow caches to warm up** - performance improves with usage
2. **Organize large file collections** into subdirectories
3. **Use preview limits** for very large files
4. **Monitor memory usage** if working with many large files

---

## Support and Development

### Getting Help

1. **Check this user guide** for common solutions
2. **Enable debug logging** to understand what's happening
3. **Review the troubleshooting section** for specific issues
4. **Check the GitHub repository** for known issues and updates

### Contributing

The project welcomes contributions:
- **Bug reports**: Include debug logs and reproduction steps
- **Feature requests**: Describe use cases and security implications
- **Code contributions**: Follow the security and testing guidelines
- **Documentation**: Help improve this guide and API documentation

### Security Reporting

For security vulnerabilities:
- **Do not** create public issues
- **Contact** maintainers directly
- **Provide** detailed reproduction steps
- **Include** potential impact assessment