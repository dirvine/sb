use anyhow::Result;
use std::collections::HashMap;

/// A sample struct to test syntax highlighting
#[derive(Debug, Clone)]
struct Config {
    name: String,
    port: u16,
    enabled: bool,
}

impl Config {
    pub fn new(name: &str, port: u16) -> Self {
        Self {
            name: name.to_string(),
            port,
            enabled: true,
        }
    }

    pub fn is_valid(&self) -> bool {
        !self.name.is_empty() && self.port > 0
    }
}

fn main() -> Result<()> {
    let mut configs: HashMap<String, Config> = HashMap::new();

    // Create some configurations
    let config1 = Config::new("server1", 8080);
    let config2 = Config::new("server2", 9000);

    configs.insert("web".to_string(), config1);
    configs.insert("api".to_string(), config2);

    for (key, config) in configs.iter() {
        println!("Config '{}': {:?}", key, config);

        if config.is_valid() {
            println!("  ✓ Valid configuration");
        } else {
            println!("  ✗ Invalid configuration");
        }
    }

    // Match statement
    let port = match configs.get("web") {
        Some(config) => config.port,
        None => 80,
    };

    println!("Using port: {}", port);
    Ok(())
}
