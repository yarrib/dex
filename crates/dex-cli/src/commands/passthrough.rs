//! Pass-through command support — delegates to external CLIs defined in dex.toml.

use std::path::PathBuf;
use std::process::Command;

use dex_core::DexError;
use dex_core::config::load_project_config;

/// Run a pass-through command. `args[0]` is the subcommand name, rest are forwarded.
pub fn run(args: Vec<String>) -> Result<(), DexError> {
    let Some(cmd_name) = args.first() else {
        return Err(DexError::Config(dex_core::error::ConfigError::Invalid(
            "no subcommand provided".into(),
        )));
    };

    // Try loading project config for passthrough definitions.
    let config = load_project_config(&PathBuf::from("dex.toml")).ok();

    let passthrough = config
        .as_ref()
        .and_then(|c| c.passthrough.get(cmd_name.as_str()));

    let target_command = match passthrough {
        Some(spec) => spec.command.clone(),
        None => {
            return Err(DexError::Config(dex_core::error::ConfigError::Invalid(
                format!("unknown command '{cmd_name}'. Not a built-in command or passthrough."),
            )));
        }
    };

    let extra_args = &args[1..];

    let status = Command::new(&target_command)
        .args(extra_args)
        .stdin(std::process::Stdio::inherit())
        .stdout(std::process::Stdio::inherit())
        .stderr(std::process::Stdio::inherit())
        .status()
        .map_err(|source| DexError::Io {
            path: PathBuf::from(&target_command),
            source,
        })?;

    if !status.success() {
        std::process::exit(status.code().unwrap_or(1));
    }

    Ok(())
}
