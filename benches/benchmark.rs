use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use std::fs;
use std::path::PathBuf;
use tempfile::TempDir;

fn create_test_files(dir: &TempDir, count: usize) -> Vec<PathBuf> {
    let mut files = Vec::new();
    for i in 0..count {
        let file_path = dir.path().join(format!("test_{}.md", i));
        let content = format!("# Test File {}\n\nThis is test content for file {}.", i, i);
        fs::write(&file_path, content).unwrap();
        files.push(file_path);
    }
    files
}

fn benchmark_file_reading(c: &mut Criterion) {
    let temp_dir = TempDir::new().unwrap();
    let files = create_test_files(&temp_dir, 100);

    let mut group = c.benchmark_group("file_reading");

    for size in [10, 50, 100].iter() {
        group.bench_with_input(BenchmarkId::from_parameter(size), size, |b, &size| {
            b.iter(|| {
                for i in 0..size {
                    let content = fs::read_to_string(&files[i]).unwrap();
                    black_box(content);
                }
            });
        });
    }

    group.finish();
}

fn benchmark_markdown_parsing(c: &mut Criterion) {
    use pulldown_cmark::{Options, Parser};

    let markdown_samples = vec![
        ("small", "# Title\n\nParagraph"),
        ("medium", include_str!("../README.md")),
        ("large", &"# Large Document\n\n".repeat(1000)),
    ];

    let mut group = c.benchmark_group("markdown_parsing");

    for (name, content) in markdown_samples {
        group.bench_with_input(BenchmarkId::from_parameter(name), &content, |b, content| {
            b.iter(|| {
                let mut options = Options::empty();
                options.insert(Options::ENABLE_STRIKETHROUGH);
                options.insert(Options::ENABLE_TABLES);

                let parser = Parser::new_ext(content, options);
                let events: Vec<_> = parser.collect();
                black_box(events);
            });
        });
    }

    group.finish();
}

fn benchmark_syntax_highlighting(c: &mut Criterion) {
    use syntect::easy::HighlightLines;
    use syntect::highlighting::{Style, ThemeSet};
    use syntect::parsing::SyntaxSet;

    let ss = SyntaxSet::load_defaults_newlines();
    let ts = ThemeSet::load_defaults();
    let theme = &ts.themes["base16-ocean.dark"];

    let code_samples = vec![
        ("rust_small", "fn main() {\n    println!(\"Hello\");\n}"),
        ("rust_medium", include_str!("../src/main.rs")),
        (
            "python",
            "def hello():\n    print('Hello, World!')\n\nif __name__ == '__main__':\n    hello()",
        ),
    ];

    let mut group = c.benchmark_group("syntax_highlighting");

    for (name, code) in code_samples {
        let syntax = ss
            .find_syntax_by_extension(if name.starts_with("rust") { "rs" } else { "py" })
            .unwrap();

        group.bench_with_input(BenchmarkId::from_parameter(name), &code, |b, code| {
            b.iter(|| {
                let mut h = HighlightLines::new(syntax, theme);
                let mut highlighted = Vec::new();

                for line in code.lines() {
                    let ranges: Vec<(Style, &str)> = h.highlight_line(line, &ss).unwrap();
                    highlighted.push(ranges);
                }

                black_box(highlighted);
            });
        });
    }

    group.finish();
}

fn benchmark_directory_traversal(c: &mut Criterion) {
    use walkdir::WalkDir;

    let temp_dir = TempDir::new().unwrap();

    // Create nested directory structure
    for i in 0..10 {
        let dir = temp_dir.path().join(format!("dir_{}", i));
        fs::create_dir(&dir).unwrap();

        for j in 0..10 {
            let file = dir.join(format!("file_{}.md", j));
            fs::write(file, format!("Content {}-{}", i, j)).unwrap();
        }
    }

    c.bench_function("directory_traversal", |b| {
        b.iter(|| {
            let entries: Vec<_> = WalkDir::new(temp_dir.path())
                .into_iter()
                .filter_map(|e| e.ok())
                .collect();
            black_box(entries);
        });
    });
}

fn benchmark_cache_operations(c: &mut Criterion) {
    use std::collections::HashMap;
    use std::time::Instant;

    struct CacheEntry {
        data: String,
        timestamp: Instant,
    }

    let mut group = c.benchmark_group("cache_operations");

    // Benchmark cache insertion
    group.bench_function("cache_insert", |b| {
        b.iter(|| {
            let mut cache: HashMap<String, CacheEntry> = HashMap::new();

            for i in 0..100 {
                let key = format!("key_{}", i);
                let entry = CacheEntry {
                    data: format!("data_{}", i),
                    timestamp: Instant::now(),
                };
                cache.insert(key, entry);
            }

            black_box(cache);
        });
    });

    // Benchmark cache lookup
    let mut cache: HashMap<String, CacheEntry> = HashMap::new();
    for i in 0..1000 {
        let key = format!("key_{}", i);
        let entry = CacheEntry {
            data: format!("data_{}", i),
            timestamp: Instant::now(),
        };
        cache.insert(key, entry);
    }

    group.bench_function("cache_lookup", |b| {
        b.iter(|| {
            for i in 0..100 {
                let key = format!("key_{}", i * 10);
                let _ = cache.get(&key);
            }
        });
    });

    group.finish();
}

fn benchmark_string_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("string_operations");

    let text = "Lorem ipsum dolor sit amet, consectetur adipiscing elit. ".repeat(100);

    group.bench_function("string_search", |b| {
        b.iter(|| {
            let count = text.matches("dolor").count();
            black_box(count);
        });
    });

    group.bench_function("string_replace", |b| {
        b.iter(|| {
            let replaced = text.replace("ipsum", "REPLACED");
            black_box(replaced);
        });
    });

    group.bench_function("string_split", |b| {
        b.iter(|| {
            let parts: Vec<&str> = text.split_whitespace().collect();
            black_box(parts);
        });
    });

    group.finish();
}

criterion_group!(
    benches,
    benchmark_file_reading,
    benchmark_markdown_parsing,
    benchmark_syntax_highlighting,
    benchmark_directory_traversal,
    benchmark_cache_operations,
    benchmark_string_operations
);
criterion_main!(benches);
