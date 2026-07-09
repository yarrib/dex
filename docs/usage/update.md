# dex update

Re-apply template changes to an already-generated project, without clobbering
your local edits.

## Synopsis

```
dex update [OPTIONS]
```

## How it works

When you scaffold a project, `dex init` records state under `.dex/` (the
template source, the ref it was generated from, and your answers), plus a
rendered baseline in `.dex/cache/baseline/`.

`dex update` renders the template twice with your recorded answers — once at the
old ref (from the baseline cache) and once at the target ref — and performs a
per-file **3-way merge** into your working tree:

- **base** = the template as it was rendered when you generated the project,
- **theirs** = the template at the target ref,
- **ours** = the files currently in your project.

Hunks that only the template changed apply automatically. Hunks where you and
the template changed the same lines are written with standard **git conflict
markers** (`<<<<<<<` / `=======` / `>>>>>>>`) for you to resolve, then commit.
Files that aren't part of the template are never touched, so unrelated local
work never conflicts. `dex update` exits `0` even when there are conflicts —
they're a normal, expected outcome.

## Options

| Option | Default | Description |
|---|---|---|
| `--ref` | latest | Target template ref (tag, commit, or version). Defaults to the latest available for the source. |
| `--dry-run` | — | Preview the changes without writing anything. |
| `--no-prompt` | — | Use defaults for any variables introduced since you generated the project. |
| `--dir`, `-d` | `.` | Project directory to update. |

## Examples

```bash
# Pull the latest template changes into the current project
dex update

# See what would change first
dex update --dry-run

# Move to a specific template version
dex update --ref v1.4.0
```

## Update hooks

A template can declare commands to run around an update in its `template.toml`:

```toml
[hooks]
pre_update  = "git stash --keep-index"
post_update = "uv sync"
```

These are copied into `.dex/manifest.toml` at generation time (and are
user-editable there). `pre_update` runs before anything is planned;
`post_update` runs after changes are applied. Both are rendered with your
answers, prompt for confirmation unless `--no-prompt` is passed, and are
non-fatal — a failing hook warns rather than aborting.

## Offline

Updates work offline for directory and embedded templates. For remote
templates, `dex update` fetches when it can and otherwise resolves the target
from the local cache clone; it only errors if the requested ref isn't available
locally.

## State layout

```
.dex/
  manifest.toml        # template source, ref, answers, hooks — commit this
  history.toml         # append-only log of applied updates — commit this
  .gitignore           # ignores cache/
  cache/baseline/      # rendered baseline for the recorded ref (not committed)
```
