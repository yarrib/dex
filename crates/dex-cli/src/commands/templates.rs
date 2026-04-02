//! `dex templates` — list and inspect available templates.

use clap::{Args, Subcommand};
use console::style;

use dex_core::DexError;
use dex_core::config::{load_dex_config, resolve_remote};
use dex_core::template::TemplateSource;
use dex_core::template::registry::{list_templates, load_template};

use crate::output;

#[derive(Args)]
pub struct TemplatesArgs {
    #[command(subcommand)]
    pub command: TemplatesCommand,
}

#[derive(Subcommand)]
pub enum TemplatesCommand {
    /// List all available templates.
    List(ListArgs),
    /// Show variables and details for a specific template.
    Show(ShowArgs),
}

#[derive(Args)]
pub struct ListArgs {
    /// Show all template variables inline.
    #[arg(long)]
    verbose: bool,
}

#[derive(Args)]
pub struct ShowArgs {
    /// Template name to inspect.
    name: String,
}

pub fn run(args: TemplatesArgs) -> Result<(), DexError> {
    match args.command {
        TemplatesCommand::List(a) => run_list(a),
        TemplatesCommand::Show(a) => run_show(a),
    }
}

fn run_list(args: ListArgs) -> Result<(), DexError> {
    let sources = collect_sources()?;

    println!("\n{}\n", style("Available templates:").bold());

    let mut any = false;
    for (label, source) in &sources {
        let mut metas = list_templates(source)?;
        if metas.is_empty() {
            continue;
        }
        metas.sort_by(|a, b| a.name.cmp(&b.name));

        println!("  {}  {}", style("●").dim(), style(label).dim());

        for meta in &metas {
            println!(
                "    {}  {}",
                style(&meta.name).cyan().bold(),
                style(&meta.description).dim(),
            );

            if args.verbose {
                if let Ok(tmpl) = load_template(source, &meta.name) {
                    for var in &tmpl.variables {
                        let default = var
                            .default
                            .as_ref()
                            .map(|v| format!(" (default: {v})"))
                            .unwrap_or_default();
                        println!(
                            "      {} {}{}",
                            style(format!("--{}", var.name)).yellow(),
                            style(&var.prompt).dim(),
                            style(default).dim(),
                        );
                    }
                }
                println!();
            }
        }

        println!();
        any = true;
    }

    if !any {
        output::print_dim("No templates found.");
    }

    println!(
        "Run {} to scaffold from a template.\n",
        style("dex init -t <name>").cyan()
    );

    Ok(())
}

fn run_show(args: ShowArgs) -> Result<(), DexError> {
    let sources = collect_sources()?;

    for (_, source) in &sources {
        if let Ok(tmpl) = load_template(source, &args.name) {
            println!(
                "\n{} {}\n",
                style("Template:").bold(),
                style(&tmpl.meta.name).cyan().bold()
            );
            println!(
                "  {}  {}",
                style("Description:").dim(),
                tmpl.meta.description
            );
            println!("  {}       {}", style("Version:").dim(), tmpl.meta.version);

            if !tmpl.variables.is_empty() {
                println!("\n  {}", style("Variables:").bold());
                for var in &tmpl.variables {
                    let required = if var.required { " (required)" } else { "" };
                    let default = var
                        .default
                        .as_ref()
                        .map(|v| format!("  default: {v}"))
                        .unwrap_or_default();
                    println!(
                        "    {}{}{}",
                        style(format!("{:<20}", var.name)).cyan(),
                        style(&var.prompt).dim(),
                        style(format!("{required}{default}")).dim(),
                    );
                }
            }

            if !tmpl.suggested_skills.is_empty() {
                println!(
                    "\n  {}  {}",
                    style("Suggested skills:").dim(),
                    style(tmpl.suggested_skills.join(", ")).cyan()
                );
            }

            println!(
                "\n  Run {} to scaffold.\n",
                style(format!("dex init -t {}", args.name)).cyan()
            );
            return Ok(());
        }
    }

    Err(DexError::Config(dex_core::error::ConfigError::Invalid(
        format!("template '{}' not found", args.name),
    )))
}

fn collect_sources() -> Result<Vec<(String, TemplateSource)>, DexError> {
    let mut sources: Vec<(String, TemplateSource)> =
        vec![("built-in".to_string(), TemplateSource::Embedded)];

    let config = load_dex_config();

    if let Some(dir) = &config.templates_dir {
        sources.push((
            dir.display().to_string(),
            TemplateSource::Directory(dir.clone()),
        ));
    }

    for remote in &config.remotes {
        match resolve_remote(remote, false) {
            Ok(local) => {
                sources.push((remote.url.clone(), TemplateSource::Directory(local)));
            }
            Err(e) => {
                output::print_warning(&format!("could not resolve remote '{}': {e}", remote.name));
            }
        }
    }

    Ok(sources)
}
