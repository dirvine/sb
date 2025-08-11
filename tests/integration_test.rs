use std::fs;
use std::path::PathBuf;
use tempfile::TempDir;

#[test]
fn test_app_initialization() {
    let temp_dir = TempDir::new().unwrap();
    let app = sb::App::new(temp_dir.path().to_path_buf());

    assert!(app.opened.is_none());
    assert!(!app.show_raw_editor);
}

#[test]
fn test_file_operations() {
    let temp_dir = TempDir::new().unwrap();
    let test_file = temp_dir.path().join("test.md");

    fs::write(&test_file, "# Test Content").unwrap();

    assert!(test_file.exists());

    let content = fs::read_to_string(&test_file).unwrap();
    assert_eq!(content, "# Test Content");
}

#[test]
fn test_directory_structure() {
    let temp_dir = TempDir::new().unwrap();
    let base_path = temp_dir.path();

    // Create nested directory structure
    fs::create_dir(base_path.join("docs")).unwrap();
    fs::create_dir(base_path.join("src")).unwrap();
    fs::create_dir(base_path.join("src/tests")).unwrap();

    fs::write(base_path.join("README.md"), "# Project").unwrap();
    fs::write(base_path.join("docs/guide.md"), "# Guide").unwrap();
    fs::write(base_path.join("src/main.rs"), "fn main() {}").unwrap();
    fs::write(base_path.join("src/tests/test.rs"), "#[test]").unwrap();

    // Verify structure
    assert!(base_path.join("docs").is_dir());
    assert!(base_path.join("src").is_dir());
    assert!(base_path.join("src/tests").is_dir());
    assert!(base_path.join("README.md").is_file());
    assert!(base_path.join("docs/guide.md").is_file());
    assert!(base_path.join("src/main.rs").is_file());
    assert!(base_path.join("src/tests/test.rs").is_file());
}

#[test]
fn test_security_path_validation() {
    let temp_dir = TempDir::new().unwrap();
    let base_path = temp_dir.path();

    // Test safe paths
    let safe_path = base_path.join("safe_file.md");
    assert!(sb::validate_path(base_path, &safe_path).is_ok());

    // Test path traversal attempts
    let unsafe_path = base_path.join("../../../etc/passwd");
    assert!(sb::validate_path(base_path, &unsafe_path).is_err());
}

#[test]
fn test_file_size_limits() {
    const MAX_FILE_SIZE: u64 = 10 * 1024 * 1024; // 10MB

    // Test file size checking
    assert!(sb::check_file_size(1024).is_ok()); // 1KB - OK
    assert!(sb::check_file_size(MAX_FILE_SIZE - 1).is_ok()); // Just under limit - OK
    assert!(sb::check_file_size(MAX_FILE_SIZE + 1).is_err()); // Over limit - Error
}

#[test]
fn test_markdown_file_detection() {
    let md_files = vec![
        PathBuf::from("test.md"),
        PathBuf::from("README.md"),
        PathBuf::from("CHANGELOG.MD"),
        PathBuf::from("notes.markdown"),
    ];

    for file in md_files {
        let ext = file.extension().and_then(|s| s.to_str()).unwrap_or("");
        assert!(ext.eq_ignore_ascii_case("md") || ext.eq_ignore_ascii_case("markdown"));
    }

    let non_md_files = vec![
        PathBuf::from("test.txt"),
        PathBuf::from("main.rs"),
        PathBuf::from("style.css"),
    ];

    for file in non_md_files {
        let ext = file.extension().and_then(|s| s.to_str()).unwrap_or("");
        assert!(!ext.eq_ignore_ascii_case("md"));
    }
}

#[test]
fn test_code_file_detection() {
    let code_files = vec![
        (PathBuf::from("main.rs"), "rust"),
        (PathBuf::from("app.js"), "javascript"),
        (PathBuf::from("test.ts"), "typescript"),
        (PathBuf::from("script.py"), "python"),
    ];

    for (file, _expected_lang) in code_files {
        let ext = file.extension().and_then(|s| s.to_str());
        assert!(ext.is_some());

        // Verify known code extensions
        let code_extensions = ["rs", "js", "ts", "py", "go", "java", "cpp", "c", "h"];
        assert!(code_extensions.contains(&ext.unwrap()));
    }
}

#[test]
fn test_config_defaults() {
    let config = sb::Config::default();

    assert_eq!(config.max_file_size(), 10 * 1024 * 1024);
    assert_eq!(config.max_preview_size(), 1024 * 1024);
    assert_eq!(config.cache_ttl_seconds(), 60);
    assert_eq!(config.max_cache_entries(), 100);
}

#[test]
fn test_concurrent_file_access() {
    use std::sync::Arc;
    use std::thread;

    let temp_dir = Arc::new(TempDir::new().unwrap());
    let file_path = temp_dir.path().join("concurrent.txt");

    // Create initial file
    fs::write(&file_path, "Initial content").unwrap();

    let mut handles = vec![];

    // Spawn multiple threads to read the file
    for i in 0..5 {
        let path = file_path.clone();
        let handle = thread::spawn(move || {
            let content = fs::read_to_string(&path).unwrap();
            assert!(!content.is_empty());
            format!("Thread {} read: {}", i, content.len())
        });
        handles.push(handle);
    }

    // Wait for all threads
    for handle in handles {
        let result = handle.join().unwrap();
        assert!(result.contains("read:"));
    }
}
