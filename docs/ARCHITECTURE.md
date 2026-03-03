# dex — Architecture

## 1. System Overview

```
┌─────────────────────────────────────────────────────────┐
│                   User / Org CLI                        │
│  (acme-dex, myorg-ops, or just `dex`)                   │
│  Python · click · pyproject.toml [project.scripts]      │
├─────────────────────────────────────────────────────────┤
│                  dex Python Package                     │
│  dex.cli      — click Group, create_cli(), passthrough  │
│  dex.ext      — extension API for templates, hooks      │
│  dex.config   — config loading / merging (Python-side)  │
├─────────────────────────────────────────────────────────┤
│              dex._core  (PyO3 bindings)                 │
│  Python ↔ Rust FFI boundary                             │
│  maturin-built cdylib                                   │
├─────────────────────────────────────────────────────────┤
│                    dex-core (Rust)                       │
│  config     — TOML parsing, schema validation           │
│  template   — engine (minijinja), registry, variables   │
│  scaffold   — directory creation, file rendering        │
├─────────────────────────────────────────────────────────┤
│                   External CLIs                         │
│  databricks · az · aws · git · uv                       │
└─────────────────────────────────────────────────────────┘
```

Data flows **downward** for operations (CLI → core → filesystem/subprocess).
Data flows **upward** for results and errors (core → Python → user).

## 2. Repository Layout

```
dex/
├── Cargo.toml                     # workspace root
├── Cargo.lock
├── pyproject.toml                 # maturin config (builds dex-py crate)
├── CLAUDE.md                      # development rules
├── LICENSE
├── README.md
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
│   └── dex-py/                    # PyO3 bindings (cdylib)
│       ├── Cargo.toml
│       └── src/
│           └── lib.rs             # #[pymodule] exposing dex-core
│
├── python/                        # Python package source
│   └── dex/
│       ├── __init__.py            # public API
│       ├── _core.pyi              # type stubs for the Rust extension
│       ├── cli.py                 # click CLI: create_cli(), DexGroup
│       ├── passthrough.py         # pass-through command support
│       └── ext.py                 # extension API (template registration, hooks)
│
└── templates/                     # built-in templates (embedded at compile time)
    └── default/
        ├── template.toml
        └── files/
            ├── pyproject.toml.j2
            ├── README.md.j2
            └── ...
```

## 3. Crate Responsibilities

### 3.1. dex-core

The library crate. **All business logic. No UI. No Python dependencies.**

This crate is the foundation. It can be used independently of Python for testing,
benchmarking, or a future standalone Rust binary.

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

### 3.2. dex-py

The FFI bridge. **Thin as possible.** Translates between Python types and Rust types,
converts errors to Python exceptions, and delegates all logic to dex-core.

**Dependencies:**
- `pyo3` (with `extension-module` feature)
- `dex-core` (workspace dependency)

**Exposes to Python as `dex._core`:**

```python
# Template rendering
def render_template(template_str: str, variables: dict[str, Any]) -> str: ...

# Full scaffold operation
def scaffold_project(
    template_path: str,
    target_dir: str,
    variables: dict[str, Any],
) -> ScaffoldResult: ...

# Config parsing
def parse_template_manifest(path: str) -> TemplateManifest: ...
def load_project_config(path: str) -> ProjectConfig: ...

# Template listing
def list_embedded_templates() -> list[TemplateMeta]: ...
```

### 3.3. Python Package (`python/dex/`)

The user-facing layer. **All CLI, UX, and extensibility logic lives here.**

This is where click commands are defined, pass-throughs are configured, and the
extension API is exposed. The Python layer handles:

- Interactive prompts (via click or rich)
- Terminal output formatting
- Plugin/entry-point discovery
- Pass-through subprocess delegation
- CLI composition (`create_cli()`)

## 4. Key Abstractions

### 4.1. Rust Side (dex-core)

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
    pub profiles: HashMap<String, ProfileSpec>,
    pub passthroughs: HashMap<String, PassthroughSpec>,
}
```

### 4.2. Python Side

```python
# dex/cli.py — the extensible CLI framework

class DexGroup(click.Group):
    """click Group subclass that supports pass-throughs and plugin discovery."""

    def __init__(self, passthroughs=None, **kwargs):
        super().__init__(**kwargs)
        self._passthroughs = passthroughs or {}

    def get_command(self, ctx, cmd_name):
        # 1. Built-in commands
        rv = super().get_command(ctx, cmd_name)
        if rv is not None:
            return rv
        # 2. Pass-through commands
        if cmd_name in self._passthroughs:
            return self._make_passthrough(cmd_name)
        # 3. Entry-point plugins
        return self._discover_plugin(cmd_name)

    def list_commands(self, ctx):
        builtins = super().list_commands(ctx)
        passthroughs = sorted(self._passthroughs.keys())
        plugins = self._discover_all_plugins()
        return builtins + passthroughs + plugins


def create_cli(
    name="dex",
    templates_dir=None,
    config_defaults=None,
    passthroughs=None,
) -> DexGroup:
    """Factory for creating a dex CLI instance.

    Teams call this to build their org-specific CLI:

        cli = create_cli(name="acme-dex", passthroughs=[...])

        @cli.command()
        def my_custom_command():
            ...
    """
    ...
```

```python
# dex/passthrough.py — pass-through command support

class PassthroughCommand(click.BaseCommand):
    """A click command that delegates to an external CLI."""

    def __init__(self, name, target_command, description=None, **kwargs):
        super().__init__(name, **kwargs)
        self.target_command = target_command
        self.help = description or f"Pass-through to `{target_command}`"

    def invoke(self, ctx):
        args = ctx.args  # everything after the command name
        result = subprocess.run(
            [self.target_command] + args,
            # inherit stdin/stdout/stderr for full interactivity
        )
        ctx.exit(result.returncode)


def passthrough(name, command, description=None):
    """Create a pass-through command spec."""
    return PassthroughSpec(name=name, command=command, description=description)
```

## 5. Data Flow

### 5.1. `dex init` Flow — Standalone Template

```
User runs: dex init --template default
                │
                ▼
    ┌── Click CLI (Python) ──┐
    │  parse args             │
    │  resolve template name  │
    └────────┬────────────────┘
             │
             ▼
    ┌── dex._core (PyO3) ───┐
    │  parse_template_manifest│
    │  → TemplateManifest     │
    │  (no [template.dabs])   │
    └────────┬────────────────┘
             │
             ▼
    ┌── Click CLI (Python) ──┐
    │  for each variable:     │
    │    prompt user (click)  │
    │  collect variables dict │
    └────────┬────────────────┘
             │
             ▼
    ┌── dex._core (PyO3) ───┐
    │  scaffold_project(      │
    │    template, dir, vars) │
    │  → ScaffoldResult       │
    └────────┬────────────────┘
             │
             ▼
    ┌── Click CLI (Python) ──┐
    │  display result         │
    │  run post-scaffold hook │
    └─────────────────────────┘
```

### 5.2. `dex init` Flow — DABs-Composite Template

When the template manifest includes `[template.dabs]`, there are two phases.
Phase 1 delegates to `databricks bundle init`. Phase 2 layers dex files on top.

```
User runs: dex init --template ml-pipeline
                │
                ▼
    ┌── Click CLI (Python) ──┐
    │  parse args             │
    │  resolve template name  │
    └────────┬────────────────┘
             │
             ▼
    ┌── dex._core (PyO3) ───┐
    │  parse_template_manifest│
    │  → TemplateManifest     │
    │  (has [template.dabs])  │
    └────────┬────────────────┘
             │
             ▼
    ┌── Click CLI (Python) ──┐
    │  for each variable:     │
    │    prompt user (click)  │
    │  collect variables dict │
    └────────┬────────────────┘
             │
             ▼  Phase 1: DABs scaffold (Python orchestrates)
    ┌── Click CLI (Python) ──────────────────────────┐
    │  map dex vars → DABs vars via variable_map      │
    │  write temp config JSON                         │
    │  subprocess.run([                               │
    │    "databricks", "bundle", "init", <source>,    │
    │    "--output-dir", <target>,                     │
    │    "--config-file", <tmp.json>                   │
    │  ])                                             │
    │  → DABs Go templates render → databricks.yml,   │
    │    notebooks, src/, etc.                        │
    └────────┬────────────────────────────────────────┘
             │
             ▼  Phase 2: dex layer on top
    ┌── dex._core (PyO3) ───┐
    │  scaffold_project(      │
    │    template, dir, vars) │
    │  renders dex files/     │
    │  → dex.toml, CI, tasks  │
    │  (skips existing files  │
    │   unless overwrite=true)│
    └────────┬────────────────┘
             │
             ▼
    ┌── Click CLI (Python) ──┐
    │  display combined result│
    │  run post-scaffold hook │
    └─────────────────────────┘
```

Key points:

- Phase 1 is pure Python (subprocess to `databricks` CLI). No Rust involved.
- Phase 2 is pure Rust (file rendering via dex-core). No subprocess.
- The Python layer orchestrates both phases and owns all user interaction.
- DABs variables are mapped from dex variables via `[template.dabs.variable_map]`,
  so the user only answers prompts once.
- If `databricks` is not installed, dex raises a clear error before Phase 1.

### 5.3. Pass-through Flow

```
User runs: dex db clusters list --output json
                │
                ▼
    ┌── Click CLI (Python) ──────┐
    │  DexGroup.get_command("db") │
    │  → PassthroughCommand       │
    │  delegate to subprocess:    │
    │  databricks clusters list   │
    │            --output json    │
    └─────────────────────────────┘
```

Pass-throughs never touch Rust. They are pure Python subprocess delegation.

## 6. Error Handling

### 6.1. Rust (dex-core)

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

#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
    #[error("config file not found: {0}")]
    NotFound(PathBuf),

    #[error("invalid config: {0}")]
    Invalid(String),

    #[error("parse error: {0}")]
    Parse(#[from] toml::de::Error),
}

#[derive(Debug, thiserror::Error)]
pub enum TemplateError {
    #[error("template not found: {0}")]
    NotFound(String),

    #[error("invalid template manifest: {0}")]
    InvalidManifest(String),

    #[error("missing required variable: {0}")]
    MissingVariable(String),

    #[error("variable validation failed: {name}: {message}")]
    ValidationFailed { name: String, message: String },
}
```

### 6.2. PyO3 Boundary (dex-py)

Rust errors are converted to Python exceptions:

```rust
use pyo3::exceptions::PyValueError;

impl From<DexError> for PyErr {
    fn from(err: DexError) -> PyErr {
        match err {
            DexError::Config(e) => PyValueError::new_err(e.to_string()),
            DexError::Template(e) => PyValueError::new_err(e.to_string()),
            // ...
        }
    }
}
```

Custom exception classes can be added later if needed.

### 6.3. Python (CLI layer)

The click CLI catches exceptions and renders them with formatting:

```python
@cli.command()
def init(template, directory, no_prompt):
    try:
        manifest = _core.parse_template_manifest(template_path)
        ...
    except ValueError as e:
        click.secho(f"Error: {e}", fg="red", err=True)
        raise SystemExit(1)
```

## 7. Testing Strategy

### 7.1. Rust Tests (dex-core)

- **Unit tests** in each module (`#[cfg(test)] mod tests`)
  - Config parsing: valid/invalid TOML, edge cases
  - Template manifest parsing: all variable types, validation patterns
  - Rendering: variable substitution, conditionals, loops
  - Scaffold: file creation, path interpolation, conditional inclusion

- **Integration tests** (`crates/dex-core/tests/`)
  - End-to-end scaffold: template directory → rendered output
  - Snapshot tests using `insta` crate: assert rendered output matches expected

### 7.2. Python Tests

- **Unit tests** for CLI commands, pass-through logic, config merging
- **Integration tests**: invoke CLI via `click.testing.CliRunner`
- **Snapshot tests**: compare scaffolded output against expected directories

### 7.3. Cross-boundary Tests

- PyO3 roundtrip tests: Python → Rust → Python, verify types and errors
- These live in `tests/` at the repo root

## 8. Build & Distribution

### 8.1. Development

```bash
# Build Rust crates
cargo build

# Build Python package (includes Rust compilation)
maturin develop

# Run Rust tests
cargo test

# Run Python tests
pytest

# Run all
cargo test && maturin develop && pytest
```

### 8.2. Release

```bash
# Build wheels for distribution
maturin build --release

# Publish to PyPI
maturin publish
```

### 8.3. CI

- Rust: `cargo clippy`, `cargo fmt --check`, `cargo test`
- Python: `ruff check`, `ruff format --check`, `pytest`
- Cross: `maturin develop && pytest`
- Matrix: Linux x86_64, macOS ARM64, Windows x86_64

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
| `pyo3`        | Python bindings            | FFI to Python (dex-py crate only)       |

### Python

| Package       | Purpose                    | Justification                           |
|---------------|----------------------------|-----------------------------------------|
| `click`       | CLI framework              | Extensible, composable, mature          |
| `rich`        | Terminal formatting        | Beautiful output, tables, spinners      |

Minimal dependencies. Each must justify its presence.
