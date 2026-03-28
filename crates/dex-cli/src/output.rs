//! Terminal output helpers — colors, formatting, status messages.

use console::style;

/// Print an error message to stderr.
pub fn print_error(msg: &str) {
    eprintln!("{} {msg}", style("Error:").red().bold());
}

/// Print a warning message to stderr.
pub fn print_warning(msg: &str) {
    eprintln!("{} {msg}", style("Warning:").yellow().bold());
}

/// Print a dim/muted message.
pub fn print_dim(msg: &str) {
    println!("{}", style(msg).dim());
}

/// Print a list of created files.
pub fn print_files_created(files: &[impl AsRef<std::path::Path>]) {
    println!(
        "\n{}",
        style(format!("Scaffolded {} files:", files.len())).green()
    );
    for f in files {
        println!("  {}", f.as_ref().display());
    }
    println!();
}
