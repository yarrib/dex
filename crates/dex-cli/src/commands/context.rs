//! `dex context` — build and refresh the project-memory knowledge graph.
//!
//! Wraps `dex_core::context_graph`: reads git history, classifies commits,
//! stitches `[[wikilink]]` edges, and writes `.context/wiki/` (per-commit nodes
//! plus INDEX.md) and `.context/USER_MANUAL.md`. The `/project-memory-engine`
//! skill drives this command and adds semantic enrichment on top.

use std::path::PathBuf;

use clap::{Args, Subcommand};
use console::style;

use dex_core::context_graph::{SyncOptions, sync};
use dex_core::error::DexError;

use crate::output;

#[derive(Args)]
pub struct ContextArgs {
    #[command(subcommand)]
    pub cmd: ContextCommand,
}

#[derive(Subcommand)]
pub enum ContextCommand {
    /// Build or refresh the project-memory graph under `.context/wiki/`.
    Sync(SyncArgs),
}

#[derive(Args)]
pub struct SyncArgs {
    /// Repository root to analyze.
    #[arg(short, long, default_value = ".")]
    dir: String,

    /// Regenerate every node from scratch instead of only adding new commits.
    #[arg(long)]
    rebuild: bool,

    /// Only consider the most recent N commits (useful for a first run).
    #[arg(long, value_name = "N")]
    limit: Option<usize>,
}

pub fn run(args: ContextArgs) -> Result<(), DexError> {
    match args.cmd {
        ContextCommand::Sync(a) => run_sync(a),
    }
}

fn run_sync(args: SyncArgs) -> Result<(), DexError> {
    let root = PathBuf::from(&args.dir);
    let opts = SyncOptions {
        rebuild: args.rebuild,
        limit: args.limit,
    };

    println!(
        "\n{} {}\n",
        style("dex context sync").bold(),
        style(if args.rebuild {
            "(full rebuild)"
        } else {
            "(incremental)"
        })
        .dim()
    );

    let report = sync(&root, &opts)?;

    // Per-area breakdown.
    for (area, count) in &report.area_counts {
        println!(
            "  {} {} {}",
            style("•").cyan(),
            style(format!("{count:>3}")).bold(),
            area.title()
        );
    }

    println!(
        "\n{} {} node{} ({} new), index at {}",
        style("✓").green().bold(),
        report.nodes_total,
        if report.nodes_total == 1 { "" } else { "s" },
        report.nodes_written,
        style(report.index_path.display()).cyan()
    );

    if report.manual_written {
        output::print_dim(&format!("  wrote {}", report.manual_path.display()));
    }
    if report.nodes_total == 0 {
        output::print_warning(
            "No commits found. Is this a git repository with history? Try without --limit.",
        );
    } else {
        output::print_dim(
            "  Open .context/wiki/ in Obsidian/Logseq/VS Code-Foam to explore the graph.",
        );
    }

    Ok(())
}
