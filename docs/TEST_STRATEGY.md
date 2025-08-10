# Test Strategy for Saorsa Browser Security & Quality Improvements

## Test Architecture

### Directory Structure
```
tests/
├── unit/                    # Component isolation tests
│   ├── security_tests.rs   # Security validation functions
│   ├── fs_tests.rs         # File system operations
│   ├── app_tests.rs        # Application logic
│   └── preview_tests.rs    # Preview generation
├── integration/             # Component interaction tests  
│   ├── navigation_tests.rs # Directory navigation flows
│   ├── preview_tests.rs    # End-to-end preview workflows
│   └── error_handling.rs   # Error scenarios
├── security/               # Vulnerability prevention tests
│   ├── path_traversal.rs   # Path traversal attack prevention
│   ├── file_size.rs        # Resource exhaustion prevention
│   └── input_validation.rs # Input sanitization
└── performance/            # Load and stress testing
    ├── large_files.rs      # Large file handling
    ├── directory_load.rs   # Directory loading performance  
    └── memory_usage.rs     # Memory consumption tests
```

## Test Categories

### 1. Unit Tests (Target: 85% coverage)

#### Security Module Tests
```rust
#[cfg(test)]
mod security_tests {
    use super::*;
    use tempfile::TempDir;
    
    #[test]
    fn test_validate_path_prevents_traversal() {
        let temp_dir = TempDir::new().unwrap();
        let base = temp_dir.path();
        
        // Should reject path traversal attempts
        assert!(validate_path(Path::new("../../../etc/passwd"), base).is_err());
        assert!(validate_path(Path::new("..\\..\\system32"), base).is_err());
        
        // Should allow valid paths within base
        let valid_path = base.join("subdir/file.txt");
        assert!(validate_path(&valid_path, base).is_ok());
    }
    
    #[test]
    fn test_file_size_checking() {
        // Test files over MAX_FILE_SIZE are rejected
        // Test valid files are accepted
        // Test directory size calculations
    }
    
    #[test]
    fn test_safe_error_messaging() {
        // Test error messages don't leak paths
        // Test generic error responses
    }
}
```

#### File System Tests
```rust
#[cfg(test)]
mod fs_tests {
    #[test]
    fn test_safe_file_reading() {
        // Test valid files read successfully
        // Test large files rejected appropriately
        // Test non-existent files handled gracefully
        // Test binary file detection
    }
    
    #[test]
    fn test_directory_listing() {
        // Test directory contents listed safely
        // Test permission denied handled
        // Test hidden file handling
    }
}
```

### 2. Integration Tests (Target: 100% critical paths)

#### Navigation Workflow Tests
```rust
#[test]
fn test_complete_navigation_workflow() {
    // GIVEN: Application starts in safe directory
    let app = App::new();
    
    // WHEN: User navigates to subdirectory
    let result = app.navigate_to("subdir");
    
    // THEN: Navigation succeeds and contents displayed
    assert!(result.is_ok());
    assert!(!app.current_files().is_empty());
}

#[test]
fn test_malicious_navigation_blocked() {
    // GIVEN: Application running
    let app = App::new();
    
    // WHEN: User attempts path traversal
    let result = app.navigate_to("../../../etc");
    
    // THEN: Access denied with safe error
    assert!(result.is_err());
    assert_eq!(result.unwrap_err(), NavigationError::AccessDenied);
}
```

### 3. Security Tests (Target: 100% vulnerability coverage)

#### Path Traversal Prevention
```rust
#[test]
fn test_directory_traversal_attacks() {
    let malicious_paths = vec![
        "../../../etc/passwd",
        "..\\..\\..\\Windows\\System32",
        "/etc/shadow",
        "~/.ssh/id_rsa",
        "file://../../sensitive",
    ];
    
    for path in malicious_paths {
        let result = validate_path(Path::new(path), &safe_base_dir());
        assert!(result.is_err(), "Path should be rejected: {}", path);
    }
}

#[test]
fn test_symlink_traversal_prevention() {
    // Create symlink pointing outside safe area
    // Attempt to follow symlink
    // Verify access denied
}
```

#### Resource Exhaustion Prevention
```rust
#[test]
fn test_large_file_protection() {
    // Create file larger than MAX_FILE_SIZE (10MB)
    let large_file = create_large_test_file(15 * 1024 * 1024);
    
    // Attempt to read/preview
    let result = read_file_safe(&large_file);
    
    // Verify rejection without memory consumption
    assert!(result.is_err());
    assert_eq!(result.unwrap_err(), FileError::FileTooLarge);
}
```

### 4. Performance Tests (Target: Response time < 100ms)

#### Large Directory Handling
```rust
#[test]
fn test_large_directory_performance() {
    // Create directory with 1000+ files
    let large_dir = create_large_directory(1500);
    
    // Measure navigation response time
    let start = Instant::now();
    let result = list_directory(&large_dir);
    let duration = start.elapsed();
    
    // Verify performance requirements
    assert!(result.is_ok());
    assert!(duration < Duration::from_millis(100));
}

#[test]
fn test_memory_usage_bounds() {
    // Monitor memory before operation
    let initial_memory = get_memory_usage();
    
    // Perform memory-intensive operations
    let _large_preview = generate_preview(&large_file);
    
    // Verify memory usage bounded
    let final_memory = get_memory_usage();
    assert!((final_memory - initial_memory) < MAX_MEMORY_INCREASE);
}
```

## Test Environment Setup

### Test Fixtures
```rust
pub struct TestEnvironment {
    temp_dir: TempDir,
    safe_files: Vec<PathBuf>,
    malicious_paths: Vec<String>,
    large_files: Vec<PathBuf>,
}

impl TestEnvironment {
    pub fn new() -> Self {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        
        // Create various test files
        let safe_files = vec![
            create_text_file(&temp_dir, "small.txt", 1024),
            create_binary_file(&temp_dir, "image.png", 50 * 1024),
            create_markdown_file(&temp_dir, "readme.md", 2048),
        ];
        
        // Define malicious path test cases
        let malicious_paths = vec![
            "../../../etc/passwd".to_string(),
            "..\\..\\system32\\config".to_string(),
            "/root/.ssh/id_rsa".to_string(),
        ];
        
        Self { temp_dir, safe_files, malicious_paths, large_files: vec![] }
    }
}
```

## Coverage Requirements

### Minimum Thresholds
- **Overall Coverage**: 80%
- **Security Functions**: 100%
- **Error Handling**: 95%
- **Critical Paths**: 100%

### Quality Gates
- ✅ All tests must pass
- ✅ No tests use `unwrap()` or `panic!()`
- ✅ Tests are deterministic and repeatable
- ✅ Performance tests meet SLA requirements
- ✅ Security tests prevent all known vulnerabilities

## Test Execution Strategy

### Continuous Testing
```bash
# Run all tests with coverage
cargo test --all-features
cargo tarpaulin --out Html --output-dir coverage

# Security-specific test run  
cargo test security --verbose

# Performance benchmarks
cargo bench --bench performance_tests
```

### Test Data Management
- Use temporary directories for all file operations
- Clean up test artifacts automatically
- Isolate tests to prevent interference
- Use deterministic test data generation

## Success Criteria

### Functionality
- All acceptance criteria covered by tests
- Edge cases and error conditions tested
- User workflows validated end-to-end

### Security
- Path traversal attacks prevented
- Resource exhaustion attacks blocked
- Input validation comprehensive
- Error messages secure

### Performance  
- Large directory navigation < 100ms
- Memory usage bounded and predictable
- No performance regressions introduced
- Caching effectiveness validated