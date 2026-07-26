//! Output formatting utilities for the CLI.

use comfy_table::{presets::UTF8_FULL, Color, Table};
use std::fmt;

/// Print a success message.
pub fn success(msg: &str) {
    println!("\x1b[32m[ok]\x1b[0m {}", msg);
}

/// Print an error message.
pub fn error(msg: &str) {
    eprintln!("\x1b[31m[error]\x1b[0m {}", msg);
}

/// Print a warning message.
pub fn warn(msg: &str) {
    eprintln!("\x1b[33m[warn]\x1b[0m {}", msg);
}

/// Print an info message.
pub fn info(msg: &str) {
    println!("\x1b[36m[info]\x1b[0m {}", msg);
}

/// Print a section header.
pub fn header(title: &str) {
    println!();
    println!("\x1b[1;37m{}\x1b[0m", title);
    println!("{}", "-".repeat(title.len()));
}

/// Print a key-value pair.
pub fn kv(key: &str, value: &dyn fmt::Display) {
    println!("  \x1b[1m{}\x1b[0m: {}", key, value);
}

/// Create a styled table with UTF8 borders.
pub fn new_table(headers: &[&str]) -> Table {
    let mut table = Table::new();
    table.load_preset(UTF8_FULL);
    let header_row: Vec<_> = headers
        .iter()
        .map(|h| comfy_table::Cell::new(*h).fg(Color::Cyan).add_attribute(comfy_table::Attribute::Bold))
        .collect();
    table.set_header(header_row);
    table
}
