//! Saorsa Browser - Terminal Markdown Browser/Editor
//!
//! A secure, performant terminal-based file browser with markdown preview capabilities.

pub mod app;
pub mod fs;
pub mod preview;

// New modules for security and configuration
pub mod config;
pub mod security;

// Test modules
#[cfg(test)]
pub mod tests;

// Re-export commonly used types
pub use app::App;
pub use config::Config;
pub use security::{check_file_size, validate_path, SecurityError};

/// Current version of the application
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
