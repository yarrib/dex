# Installation

dex is distributed as a pre-built native binary on [GitHub Releases](https://github.com/yarrib/dex/releases).
No Python, no runtime, no toolchain required to run it.

## Install script (recommended)

The fastest way to install dex is with the installer script, which downloads the right binary for your platform:

```bash
curl -sSf https://raw.githubusercontent.com/yarrib/dex/main/install.sh | sh
```

The script will:

1. Detect your OS and CPU architecture
2. Fetch the latest release from GitHub
3. Download the correct binary
4. Place it in `~/.local/bin` (or the platform default)

After install, `dex` is available globally:

```bash
dex --help
```

> **Note:** The install script supports Linux and macOS. For Windows, use the manual install path below.

## Manual install

1. Go to the [latest release](https://github.com/yarrib/dex/releases/latest)
2. Download the binary for your platform:

| Platform | Filename |
|---|---|
| Linux x86\_64 | `dex-linux-x86_64` |
| Linux aarch64 | `dex-linux-aarch64` |
| macOS Apple Silicon | `dex-macos-aarch64` |
| macOS Intel | `dex-macos-x86_64` |
| Windows x86\_64 | `dex-windows-x86_64.exe` |

3. Make it executable and move it onto your `PATH`:

```bash
chmod +x dex-linux-x86_64
mv dex-linux-x86_64 ~/.local/bin/dex
```

## Upgrade

Re-run the install script to upgrade to the latest release:

```bash
curl -sSf https://raw.githubusercontent.com/yarrib/dex/main/install.sh | sh
```

Or download the new binary manually from [GitHub Releases](https://github.com/yarrib/dex/releases).

## Uninstall

Remove the binary from wherever you placed it:

```bash
rm ~/.local/bin/dex
```

## Build from source

Requires [Rust](https://rustup.rs/) (stable).

```bash
git clone https://github.com/yarrib/dex
cd dex
cargo build --release
# Binary at: target/release/dex
```

Install directly:

```bash
cargo install --path crates/dex-cli
```

## Requirements

No runtime dependencies. The `dex` binary is fully self-contained.

For building from source: Rust stable toolchain (`rustup update stable`).
