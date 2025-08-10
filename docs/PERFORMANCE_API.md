# Performance Module API Documentation

## Overview

The performance modules provide high-performance file operations, intelligent caching, and comprehensive monitoring for Saorsa Browser. These modules work together to deliver responsive user experience while maintaining security.

## Modules

### 1. Async File Manager (`src/async_file_manager.rs`)
### 2. Caching System (`src/cache.rs`)
### 3. Performance Monitor (`src/performance_monitor.rs`)

---

## Async File Manager

### Overview

The async file manager provides non-blocking file operations using Tokio, ensuring the UI remains responsive during I/O operations.

### `AsyncFileManager`

Main structure providing async file operations with integrated caching and security.

```rust
pub struct AsyncFileManager {
    directory_cache: DirectoryCache,
    file_cache: FilePreviewCache,
    performance_monitor: PerformanceMonitor,
}
```

#### `new() -> Self`

Creates a new async file manager with default caching settings.

**Example:**
```rust
let file_manager = AsyncFileManager::new();
```

#### `read_directory_async(path: &Path) -> AsyncFileResult<Vec<PathBuf>>`

Asynchronously reads directory contents with intelligent caching.

**Parameters:**
- `path: &Path` - Directory path to read

**Returns:**
- `Ok(Vec<PathBuf>)` - List of entries in the directory
- `Err(AsyncFileError)` - I/O error or security violation

**Performance Features:**
- **Cache-First**: Checks cache before disk access
- **Async I/O**: Non-blocking directory reading
- **Security Integration**: Path validation built-in
- **Metrics**: Operation timing and cache hit tracking

**Example:**
```rust
let entries = file_manager.read_directory_async(&dir_path).await?;
for entry in entries {
    println!("Found: {}", entry.display());
}
```

#### `get_file_preview(path: &Path, max_bytes: Option<u64>) -> AsyncFileResult<String>`

Retrieves file preview content with size limits and caching.

**Parameters:**
- `path: &Path` - File path to preview
- `max_bytes: Option<u64>` - Maximum bytes to read (default: 1MB)

**Returns:**
- `Ok(String)` - File content as UTF-8 string
- `Err(AsyncFileError)` - I/O, encoding, or security error

**Performance Optimizations:**
- **Streaming**: Large files read in chunks
- **Cache Strategy**: Small files cached, large files streamed
- **UTF-8 Validation**: Encoding validation with fallback
- **Size Limits**: Prevents memory exhaustion

**Example:**
```rust
// Preview first 1KB of file
let preview = file_manager
    .get_file_preview(&file_path, Some(1024))
    .await?;
    
println!("Preview: {}", preview);
```

#### `write_file_async(path: &Path, content: &str) -> AsyncFileResult<()>`

Asynchronously writes content to file with security validation.

**Parameters:**
- `path: &Path` - Destination file path
- `content: &str` - Content to write

**Returns:**
- `Ok(())` - Write successful
- `Err(AsyncFileError)` - Write failed or security violation

**Safety Features:**
- **Path Validation**: Prevents writes outside safe boundaries
- **Atomic Writes**: Temporary file with atomic rename
- **Cache Invalidation**: Updates caches after write
- **Backup**: Optional backup of original file

---

## Caching System

### Overview

Multi-layer caching system providing intelligent caching with TTL expiration and LRU eviction.

### `DirectoryCache`

Caches directory listings to improve navigation performance.

```rust
pub struct DirectoryCache {
    entries: Arc<RwLock<HashMap<PathBuf, CacheEntry<Vec<PathBuf>>>>>,
    default_ttl: Duration,
    max_entries: usize,
}
```

#### Configuration
- **Default TTL**: 30 seconds
- **Max Entries**: 1000 directories
- **Eviction Policy**: LRU with TTL expiration

#### `new() -> Self`

Creates directory cache with default settings.

#### `get(path: &Path) -> Option<Vec<PathBuf>>`

Retrieves cached directory contents if valid.

**Example:**
```rust
let cache = DirectoryCache::new();
if let Some(entries) = cache.get(&path) {
    // Use cached entries
} else {
    // Read from disk and cache
}
```

#### `insert(path: PathBuf, entries: Vec<PathBuf>)`

Caches directory contents with TTL.

#### `clear()`

Clears all cached entries.

#### `cache_info() -> CacheInfo`

Returns cache statistics for monitoring.

### `FilePreviewCache`

Caches file preview content with intelligent size-based policies.

```rust
pub struct FilePreviewCache {
    entries: Arc<RwLock<HashMap<PathBuf, CacheEntry<String>>>>,
    default_ttl: Duration,
    max_entries: usize,
    max_content_size: usize,
}
```

#### Configuration
- **Default TTL**: 60 seconds
- **Max Entries**: 500 files
- **Max Content Size**: 100KB per file
- **Total Memory Limit**: ~50MB

#### Cache Strategy
- **Small Files**: Cached in memory for fast access
- **Large Files**: Not cached to prevent memory pressure
- **Frequent Access**: LRU promotes commonly used files

---

## Performance Monitor

### Overview

Comprehensive performance monitoring with metrics collection and analysis.

### `PerformanceMonitor`

Tracks operation timings, cache performance, and system metrics.

```rust
pub struct PerformanceMonitor {
    operation_times: HashMap<String, Vec<Duration>>,
    cache_hits: u64,
    cache_misses: u64,
    total_operations: u64,
}
```

#### `new() -> Self`

Creates a new performance monitor.

#### `start_operation(operation_name: &str) -> OperationTimer`

Starts timing an operation.

**Example:**
```rust
let monitor = PerformanceMonitor::new();
let timer = monitor.start_operation("read_directory");
// ... perform operation ...
timer.finish(); // Automatically records duration
```

#### `record_cache_hit()` / `record_cache_miss()`

Records cache performance metrics.

#### `get_statistics() -> PerformanceStats`

Returns comprehensive performance statistics.

```rust
pub struct PerformanceStats {
    pub average_operation_time: HashMap<String, Duration>,
    pub cache_hit_rate: f64,
    pub total_operations: u64,
    pub operations_per_second: f64,
}
```

#### `log_performance_summary()`

Logs performance summary using structured logging.

**Example Output:**
```
INFO performance_summary operation_count=1250 cache_hit_rate=0.87 avg_read_time_ms=12.5
```

---

## Error Types

### `AsyncFileError`

Comprehensive error handling for async operations.

```rust
#[derive(Error, Debug)]
pub enum AsyncFileError {
    #[error("Security violation: {0}")]
    Security(#[from] SecurityError),
    
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    
    #[error("UTF-8 encoding error: {0}")]
    Encoding(#[from] std::string::FromUtf8Error),
    
    #[error("File too large: {0} bytes")]
    FileTooLarge(u64),
    
    #[error("Operation cancelled")]
    Cancelled,
}
```

---

## Integration Example

### Complete File Operation with Performance Monitoring

```rust
use crate::async_file_manager::AsyncFileManager;
use crate::performance_monitor::PerformanceMonitor;
use std::path::Path;

async fn process_directory(path: &Path) -> Result<(), Box<dyn std::error::Error>> {
    let file_manager = AsyncFileManager::new();
    let monitor = PerformanceMonitor::new();
    
    // Time the directory read operation
    let timer = monitor.start_operation("read_directory");
    let entries = file_manager.read_directory_async(path).await?;
    timer.finish();
    
    // Process each file
    for entry in entries {
        if entry.is_file() {
            let timer = monitor.start_operation("get_preview");
            let preview = file_manager
                .get_file_preview(&entry, Some(1024))
                .await?;
            timer.finish();
            
            println!("Preview of {}: {}", entry.display(), preview);
        }
    }
    
    // Log performance summary
    monitor.log_performance_summary();
    
    Ok(())
}
```

---

## Performance Benchmarks

### Target Performance Goals

- **Directory Navigation**: <100ms for typical directories
- **File Preview**: <50ms for files under 1MB
- **Cache Hit Rate**: >80% for typical usage patterns
- **Memory Usage**: <100MB for cache data
- **UI Responsiveness**: No blocking operations >16ms

### Benchmark Results

The `benches/file_operations.rs` provides comprehensive benchmarking:

```bash
cargo bench

# Sample output:
directory_read_cached     time:   [1.2345 ms 1.2567 ms 1.2789 ms]
directory_read_uncached   time:   [12.345 ms 12.567 ms 12.789 ms]  
file_preview_small        time:   [0.5234 ms 0.5456 ms 0.5678 ms]
file_preview_large        time:   [45.123 ms 45.456 ms 45.789 ms]
cache_hit_rate           rate:    [0.8456 0.8567 0.8678]
```

---

## Configuration and Tuning

### Environment Variables

- `SB_CACHE_TTL_SECS`: Directory cache TTL (default: 30)
- `SB_FILE_CACHE_TTL_SECS`: File cache TTL (default: 60)
- `SB_MAX_CACHE_ENTRIES`: Maximum cache entries (default: 1000)
- `SB_MAX_FILE_SIZE_MB`: Maximum file size in MB (default: 10)

### Runtime Configuration

```rust
let config = CacheConfig {
    directory_ttl: Duration::from_secs(60),
    file_ttl: Duration::from_secs(120),
    max_entries: 2000,
    max_memory_mb: 100,
};

let file_manager = AsyncFileManager::with_config(config);
```

---

## Monitoring and Observability

### Structured Logging

All performance modules integrate with the logging system:

```rust
// Automatic performance logging
info!("directory_read_completed", 
      path = %path.display(),
      duration_ms = timer.elapsed().as_millis(),
      cache_hit = cache_hit,
      entry_count = entries.len());
```

### Metrics Export

Performance metrics can be exported for external monitoring:

```rust
let stats = monitor.get_statistics();
println!("{}", serde_json::to_string(&stats)?);
```

### Health Checks

Built-in health checks for performance monitoring:

```rust
pub fn health_check() -> HealthStatus {
    let stats = monitor.get_statistics();
    
    if stats.cache_hit_rate < 0.5 {
        HealthStatus::Warning("Low cache hit rate".to_string())
    } else if stats.average_operation_time["read_directory"] > Duration::from_millis(1000) {
        HealthStatus::Critical("Slow directory reads".to_string())
    } else {
        HealthStatus::Healthy
    }
}
```