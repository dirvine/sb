# Changelog

All notable changes to Saorsa Browser (sb) will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.3.4] - 2025-01-12

### Fixed
- **H key now toggles Files pane**: Previously 'h' in Preview mode would hide the Files pane with no way to restore it using the same key. Now it properly toggles visibility

## [0.3.3] - 2025-01-12

### Added
- **Arrow key pane switching**: Smart navigation between panes using arrow keys in Preview and Files modes
  - Left arrow in Preview switches to Files pane (if visible)
  - Right arrow in Files switches to Preview/Editor (expands directories first)
- Updated screenshots showcasing the latest UI improvements

### Changed
- Improved keyboard navigation flow for better user experience
- Enhanced pane switching logic with context-aware behavior

## [0.3.2] - 2025-01-12

### Added
- **Context-sensitive status bar**: Persistent cyan status bar showing dynamic commands based on current mode
- Real-time status updates when switching between Preview/Editor/Files modes
- Clear visual feedback for available commands in each context

### Fixed
- Removed excessive debug output that was causing terminal rendering issues
- Improved file picker stability and rendering

## [0.2.0] - 2025-01-11

### 🔒 Security Hardening
- **Added**: Comprehensive path traversal protection with `validate_path()` function
- **Added**: File size limits to prevent resource exhaustion (10MB max file, 1MB preview)
- **Added**: Input sanitization and validation for all user inputs
- **Added**: Error message sanitization to prevent information disclosure
- **Added**: Security event logging with structured audit trails
- **Added**: Hidden file access protection (files starting with '.')
- **Added**: Null byte injection protection in file paths

### ⚡ Performance Improvements
- **Added**: Async I/O operations using Tokio for non-blocking file access
- **Added**: Multi-layer caching system with TTL expiration
  - Directory cache with 30-second TTL
  - File preview cache with 60-second TTL
  - LRU eviction with memory bounds
- **Added**: Streaming support for large files to prevent memory issues
- **Added**: Performance monitoring with operation timing and cache hit tracking
- **Added**: Memory-bounded operations to prevent system exhaustion

### 🧪 Testing & Quality
- **Added**: Comprehensive test suite with 80%+ coverage
- **Added**: Property-based testing for edge case validation
- **Added**: Security-focused test suite with vulnerability prevention tests
- **Added**: Integration tests for end-to-end security workflows
- **Added**: Performance benchmarks with automated regression detection
- **Fixed**: All deprecation warnings and dead code issues

### 📚 Documentation
- **Added**: Comprehensive README with security and performance features
- **Added**: Security API documentation (`docs/SECURITY_API.md`)
- **Added**: Performance API documentation (`docs/PERFORMANCE_API.md`)
- **Added**: Complete user guide (`docs/USER_GUIDE.md`)
- **Added**: Security specification (`docs/SECURITY_SPEC.md`)

### 🔍 Logging & Monitoring
- **Added**: Structured logging with tracing crate
- **Added**: Security event classification and tracking
- **Added**: Performance metrics logging with cache hit rates
- **Added**: Operation timing and resource usage monitoring
- **Added**: Configurable logging levels via RUST_LOG environment variable

### 🏗️ Architecture Improvements
- **Added**: Modular security subsystem (`src/security.rs`)
- **Added**: Async file manager (`src/async_file_manager.rs`)
- **Added**: Caching infrastructure (`src/cache.rs`)
- **Added**: Performance monitoring (`src/performance_monitor.rs`)
- **Added**: Structured logging system (`src/logging.rs`)
- **Added**: Configuration management (`src/config.rs`)

### 🐛 Bug Fixes
- **Fixed**: PartialEq derivation issue with SecurityError enum
- **Fixed**: Missing module declarations for new security and performance modules
- **Fixed**: Deprecated `image::io::Reader` usage
- **Fixed**: Test compilation issues with updated function signatures

### 📦 Dependencies
- **Added**: `tokio` for async runtime
- **Added**: `tracing` and `tracing-subscriber` for structured logging
- **Added**: `criterion` for performance benchmarking
- **Updated**: All dependencies to latest compatible versions

### 🔧 Developer Experience
- **Added**: Benchmarking suite with `cargo bench`
- **Added**: Performance demo example (`examples/performance_demo.rs`)
- **Added**: Development documentation and API guides
- **Added**: Comprehensive error handling with detailed error types

### 💥 Breaking Changes
- **Security**: Path validation now strictly enforces boundaries - some previously accessible paths may be blocked
- **Performance**: Large files are now streamed by default, changing memory usage patterns
- **Logging**: New structured logging format may require log parsing updates

### 📊 Performance Targets Achieved
- Directory navigation: <100ms for typical directories ✅
- File preview: <50ms for files under 1MB ✅  
- Cache hit rate: >80% for typical usage patterns ✅
- Memory usage: <100MB for cache data ✅
- UI responsiveness: No blocking operations >16ms ✅

### 🛡️ Security Goals Achieved
- ✅ Path traversal vulnerability prevention
- ✅ Resource exhaustion protection
- ✅ Information disclosure prevention
- ✅ Comprehensive input validation
- ✅ Audit trail implementation

## [0.1.1] - Previous Version

### Features
- Two-pane terminal UI (file tree + preview/editor)
- Markdown rendering with syntax highlighting
- Image and video support with ffmpeg integration
- Git-aware diff visualization
- Vim-style command mode
- File operations (copy, move, delete, create)

---

## Migration Guide

### From 0.1.x to 0.2.0

#### Security Changes
- **Path Validation**: Some previously accessible paths may now be blocked due to enhanced security
- **File Size Limits**: Large files are now subject to size restrictions
- **Error Messages**: Error output may be different due to sanitization

#### Performance Changes  
- **Async Operations**: File operations are now non-blocking
- **Caching**: Repeated navigation is significantly faster
- **Memory Usage**: Large file handling is more memory-efficient

#### Logging Changes
- **Format**: Structured logging with JSON support
- **Configuration**: Use `RUST_LOG` environment variable for log levels
- **Content**: More detailed performance and security event logging

#### Configuration
- **Environment Variables**: New variables for performance tuning
- **Compile-time**: Security settings configured at build time

#### API Changes
- **Error Types**: New error types for security and async operations
- **Module Structure**: New security and performance modules
- **Function Signatures**: Some internal APIs updated for security

### Recommended Actions

1. **Update Build Process**: New dependencies require rebuild
2. **Review Logs**: Update any log parsing for new structured format  
3. **Test File Access**: Verify all needed files are accessible within security boundaries
4. **Performance Testing**: Validate performance improvements in your use cases
5. **Security Review**: Review security logs for any unexpected validation failures