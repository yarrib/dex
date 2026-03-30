# dex — Architecture

## 1. System Overview

```
┌─────────────────────────────────────────────────────────┐
│                   User / Org CLI                        │
│  configured via dex.toml [passthrough] + templates/     │
├─────────────────────────────────────────────────────────┤
│                    dex-cli (Rust)                        │
│  clap        — argument parsing                         │
│  dialoguer   — interactive prompts                       │
│  console     — terminal output / styling                 │
├─────────────────────────────────────────────────────────┤
│                    dex-core (Rust)                       │
│  config     — TOML parsing, schema validation           │
│  template   — engine (minijinja), registry, variables   │
│  scaffold   — directory creation, file rendering        │
├─────────────────────────────────────────────────────────┤
│                   External CLIs                         │
│  databricks · az · aws · git                            │
└─────────────────────────────────────────────────────────┘
```

Data flows **downward** for operations (CLI → core → filesystem/subprocess).
Data flows **upward** for results and errors (core → CLI → user).

## 2. Repository Layout

```
dex/
├── Cargo.toml                     # workspace root
├── Cargo.lock
├── CLAUDE.md                      # development rules
├── LICENSE
├── README.md
├── Makefile
├── docs/
│   ├── SPEC.md                    # project specification
│   └── ARCHITECTURE.md            # this file
│
├── crates/
│   ├── dex-core/                  # Rust library — all business logic
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs             # public API surface
│   │       ├── config.rs          # TOML config parsing & merging
│   │       ├── error.rs           # error types (thiserror)
│   │       ├── template/
│   │       │   ├── mod.rs         # re-exports
│   │       │   ├── engine.rs      # minijinja Environment wrapper
│   │       │   ├── manifest.rs    # template.toml deserialization
│   │       │   ├── registry.rs    # template discovery & loading
│   │       │   └── variables.rs   # variable specs, defaults, validation
│   │       └── scaffold.rs        # orchestrates template → directory
│   │
│   └── dex-cli/                   # Binary crate — CLI, prompts, output
│       ├── Cargo.toml
│       └── src/
│           ├── main.rs            # entry point, command dispatch
│           └── commands/          # one module per subcommand
│               ├── init.rs
│               ├── agent.rs
│               └── mcp.rs
│
└── templates/                     # built-in templates (embedded at compile time)
    ├── default/
    │   ├── template.toml
    │   └── files/
    ├── dabs-package/
    │   ├── template.toml
    │   └── files/
    └── ...
```

## 3. Crate Responsibilities

### 3.1. dex-core

The library crate. **All business logic. No UI. No terminal output.**

This crate is the foundation. It can be used independently for testing, benchmarking,
or embedded in other Rust programs.

**Dependencies:**
- `serde` + `toml` — config parsing
- `minijinja` (with `loader` feature) — template rendering
- `thiserror` — typed error definitions
- `walkdir` — directory traversal
- `include_dir` — embed built-in templates at compile time
- `regex` — variable validation patterns

**Public API surface (lib.rs):**

```rust
// Config
pub fn load_project_config(path: &Path) -> Result<ProjectConfig>;

// Template operations
pub fn load_template(source: &TemplateSource) -> Result<Template>;
pub fn list_templates(sources: &[TemplateSource]) -> Result<Vec<TemplateMeta>>;

// Scaffolding
pub fn scaffold(
    template: &Template,
    target_dir: &Path,
    variables: &HashMap<String, Value>,
) -> Result<ScaffoldResult>;

// Template rendering (low-level)
pub fn render_string(template_str: &str, context: &Context) -> Result<String>;
```

### 3.2. dex-cli

The binary crate. **All user interaction lives here.**

Handles argument parsing (clap), interactive prompts (dialoguer), terminal styling
(console), progress indicators (indicatif), and error display. Delegates all business
logic to dex-core.

**Dependencies:**
- `clap` (with derive macros) — argument parsing
- `dialoguer` — interactive prompts
- `console` — terminal styling
- `indicatif` — progress spinners
- `dex-core` (workspace dependency)

## 4. Key Abstractions

### 4.1. Rust (dex-core)

```rust
/// Where templates come from
pub enum TemplateSource {
    /// Compiled into the binary
    Embedded,
    /// Filesystem directory
    Directory(PathBuf),
}

/// Template metadata (from template.toml [template] section)
pub struct TemplateMeta {
    pub name: String,
    pub description: String,
    pub version: String,
    pub min_dex_version: Option<String>,
}

/// A fully-loaded template ready for rendering
pub struct Template {
    pub meta: TemplateMeta,
    pub variables: Vec<VariableSpec>,
    pub file_rules: Vec<FileRule>,
    pub files: HashMap<PathBuf, String>,  // path → content
}

/// Variable specification from template.toml
pub struct VariableSpec {
    pub name: String,
    pub prompt: String,
    pub var_type: VariableType,
    pub default: Option<Value>,
    pub required: bool,
    pub choices: Option<Vec<String>>,
    pub validate: Option<String>,  // regex pattern
}

pub enum VariableType {
    String,
    Bool,
    Choice,
    Multi,
}

/// Conditional file inclusion / path remapping
pub struct FileRule {
    pub src: String,
    pub dest: Option<String>,
    pub condition: Option<String>,  // variable name (must be truthy)
}

/// Result of a scaffold operation
pub struct ScaffoldResult {
    pub files_created: Vec<PathBuf>,
    pub directories_created: Vec<PathBuf>,
}

/// Project config from dex.toml
pub struct ProjectConfig {
    pub project: ProjectMeta,
    pub tasks: HashMap<String, TaskSpec>,
    pub passthroughs: HashMap<String, PassthroughSpec>,
}
```

## 5. Data Flow

### 5.1. `dex init` Flow

```
User runs: dex init --template default
                │
                ▼
    ┌── dex-cli (clap) ─────────┐
    │  parse args                │
    │  resolve template name     │
    └────────┬───────────────────┘
             │
             ▼
    ┌── dex-core ───────────────┐
    │  load_template()           │
    │  → Template + VariableSpec │
    └────────┬───────────────────┘
             │
             ▼
    ┌── dex-cli (dialoguer) ────┐
    │  for each variable:        │
    │    prompt user             │
    │  collect variables HashMap │
    └────────┬───────────────────┘
             │
             ▼
    ┌── dex-core ───────────────┐
    │  scaffold(                 │
    │    template, dir, vars)    │
    │  → ScaffoldResult          │
    └────────┬───────────────────┘
             │
             ▼
    ┌── dex-cli (console) ──────┐
    │  display result            │
    └────────────────────────────┘
```

### 5.2. Pass-through Flow

```
User runs: dex db clusters list --output json
                │
                ▼
    ┌── dex-cli ────────────────────────┐
    │  load dex.toml                     │
    │  resolve passthrough "db"          │
    │  std::process::Command::new(       │
    │    "databricks")                   │
    │    .args(["clusters", "list",      │
    │           "--output", "json"])     │
    │  inherit stdin/stdout/stderr       │
    └────────────────────────────────────┘
```

Pass-throughs never touch dex-core. They are pure subprocess delegation.

## 6. Error Handling

### 6.1. dex-core

All errors use `thiserror` with structured variants:

```rust
#[derive(Debug, thiserror::Error)]
pub enum DexError {
    #[error("config error: {0}")]
    Config(#[from] ConfigError),

    #[error("template error: {0}")]
    Template(#[from] TemplateError),

    #[error("render error: {0}")]
    Render(#[from] minijinja::Error),

    #[error("I/O error: {path}: {source}")]
    Io {
        path: PathBuf,
        source: std::io::Error,
    },
}
```

No `unwrap()` or `expect()` in library code. Errors propagate with `?`.

### 6.2. dex-cli

The CLI catches errors and renders them with formatting:

```rust
fn run() -> Result<(), DexError> {
    let args = Cli::parse();
    match args.command {
        Command::Init(opts) => commands::init::run(opts)?,
        // ...
    }
    Ok(())
}

fn main() {
    if let Err(e) = run() {
        eprintln!("error: {e}");
        std::process::exit(1);
    }
}
```

## 7. Testing Strategy

### 7.1. dex-core

- **Unit tests** in each module (`#[cfg(test)] mod tests`)
  - Config parsing: valid/invalid TOML, edge cases
  - Template manifest parsing: all variable types, validation patterns
  - Rendering: variable substitution, conditionals, loops
  - Scaffold: file creation, path interpolation, conditional inclusion

- **Integration tests** (`crates/dex-core/tests/`)
  - End-to-end scaffold: template directory → rendered output
  - Snapshot tests using `insta` crate: assert rendered output matches expected

### 7.2. dex-cli

- **Integration tests**: invoke CLI via `assert_cmd` or similar
- **Snapshot tests**: compare scaffolded output against expected directories

## 8. Build & Distribution

### 8.1. Development

```bash
# Build
cargo build

# Run tests
cargo test

# Lint
cargo clippy -- -D warnings

# Format check
cargo fmt --check
```

### 8.2. Release

The CI builds native binaries for each target platform on tag push:

- Linux x86\_64
- Linux aarch64
- macOS Apple Silicon (arm64)
- macOS Intel (x86\_64)
- Windows x86\_64

Binaries are attached to the GitHub Release.

### 8.3. CI

- `cargo clippy -- -D warnings`
- `cargo fmt --check`
- `cargo test`
- Matrix: Linux x86\_64, macOS ARM64, Windows x86\_64

## 9. Dependency Policy

### Rust

| Crate         | Purpose                    | Justification                           |
|---------------|----------------------------|-----------------------------------------|
| `serde`       | Serialization              | Industry standard                       |
| `toml`        | TOML parsing               | Config format                           |
| `minijinja`   | Template rendering         | Jinja2-compatible, by Armin Ronacher    |
| `thiserror`   | Error types                | Ergonomic derive for error enums        |
| `walkdir`     | Directory traversal        | Recursive file discovery                |
| `include_dir` | Embed templates            | Zero-cost built-in templates            |
| `regex`       | Validation patterns        | Variable validation                     |
| `clap`        | Argument parsing           | Derive macros, excellent UX             |
| `dialoguer`   | Interactive prompts        | Terminal prompts, select, confirm       |
| `console`     | Terminal styling           | Colors, bold, styled output             |
| `indicatif`   | Progress indicators        | Spinners, progress bars                 |

Minimal dependencies. Each must justify its presence.
