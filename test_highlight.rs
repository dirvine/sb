use once_cell::sync::Lazy;
use syntect::{highlighting::ThemeSet, parsing::SyntaxSet};

static SYNTAX_SET: Lazy<SyntaxSet> = Lazy::new(SyntaxSet::load_defaults_newlines);
static THEME_SET: Lazy<ThemeSet> = Lazy::new(ThemeSet::load_defaults);

fn main() {
    println!("Available syntax definitions:");
    for syntax in SYNTAX_SET.syntaxes() {
        println!("  {}: extensions {:?}", syntax.name, syntax.file_extensions);
    }

    println!("\nAvailable themes:");
    for theme_name in THEME_SET.themes.keys() {
        println!("  {}", theme_name);
    }
}
