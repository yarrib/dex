#!/usr/bin/env python3
"""Generate the repo knowledge-graph page from `.context/` entity files.

Reads every `.context/*.md` file (skipping README.md), parses its frontmatter
(`id`, `title`, `kind`, `summary`, `related:` edges) and emits a single mdBook
page at `docs/knowledge-graph.md` containing:

  1. a Mermaid diagram of all entities and their relationships, and
  2. one section per entity with its summary, body and outgoing relations.

The `.context/*.md` files are the source of truth; this page is generated.
Run from the repo root:  python3 scripts/gen_context_graph.py

Stdlib only — no third-party dependencies, so it runs unchanged in CI.
"""

from __future__ import annotations

import sys
from pathlib import Path

REPO_ROOT = Path(__file__).resolve().parent.parent
CONTEXT_DIR = REPO_ROOT / ".context"
OUTPUT = REPO_ROOT / "docs" / "knowledge-graph.md"

# Order entities are grouped/rendered in, and the Mermaid class per kind.
KIND_ORDER = ["crate", "module", "concept", "artifact", "config", "meta"]


def node_id(raw: str) -> str:
    """Mermaid-safe node id (letters, digits and underscores only)."""
    return "".join(c if c.isalnum() else "_" for c in raw)


def parse_frontmatter(text: str) -> tuple[dict, str]:
    """Parse the leading `--- ... ---` frontmatter block.

    Returns (meta, body). `meta['related']` is a list of (target, rel) tuples.
    Deliberately small: it only understands the flat schema this repo uses, so
    we avoid a PyYAML dependency.
    """
    lines = text.splitlines()
    if not lines or lines[0].strip() != "---":
        raise ValueError("missing frontmatter")

    meta: dict = {"related": []}
    in_related = False
    i = 1
    while i < len(lines):
        line = lines[i]
        if line.strip() == "---":
            i += 1
            break
        if line.rstrip() == "related:":
            in_related = True
            i += 1
            continue
        if in_related and line.lstrip().startswith("- "):
            item = line.lstrip()[2:].strip()
            if ":" in item:
                target, rel = item.split(":", 1)
                meta["related"].append((target.strip(), rel.strip()))
            else:
                meta["related"].append((item.strip(), "related"))
            i += 1
            continue
        in_related = False
        if ":" in line:
            key, value = line.split(":", 1)
            meta[key.strip()] = value.strip()
        i += 1

    body = "\n".join(lines[i:]).strip()
    return meta, body


def load_entities() -> list[dict]:
    entities = []
    for path in sorted(CONTEXT_DIR.glob("*.md")):
        if path.name.lower() == "readme.md":
            continue
        meta, body = parse_frontmatter(path.read_text(encoding="utf-8"))
        meta.setdefault("id", path.stem)
        meta.setdefault("title", meta["id"])
        meta.setdefault("kind", "concept")
        meta.setdefault("summary", "")
        meta["body"] = body
        meta["_file"] = path.name
        entities.append(meta)
    return entities


def kind_sort_key(entity: dict) -> tuple[int, str]:
    kind = entity.get("kind", "concept")
    rank = KIND_ORDER.index(kind) if kind in KIND_ORDER else len(KIND_ORDER)
    return (rank, entity["id"])


def build_mermaid(entities: list[dict]) -> str:
    by_id = {e["id"]: e for e in entities}
    lines = ["```mermaid", "graph LR"]

    # Declare internal nodes (with clickable links to their sections below).
    for e in sorted(entities, key=kind_sort_key):
        nid = node_id(e["id"])
        title = e["title"].replace('"', "'")
        lines.append(f'  {nid}["{title}"]:::{e["kind"]}')

    # Declare external nodes — referenced as a target but with no `.context` file.
    external: dict[str, None] = {}
    for e in entities:
        for target, _rel in e["related"]:
            if target not in by_id and target not in external:
                external[target] = None
    for target in sorted(external):
        nid = node_id(target)
        lines.append(f'  {nid}["{target}"]:::external')

    lines.append("")

    # Edges. Collapse reciprocal pairs (e.g. A "contains" B and B "part-of" A)
    # to a single edge so the diagram stays readable. The full relationship
    # lists are preserved verbatim in the per-entity text sections below.
    seen_pairs: set[frozenset[str]] = set()
    for e in sorted(entities, key=kind_sort_key):
        src = e["id"]
        for target, rel in e["related"]:
            pair = frozenset((src, target))
            if pair in seen_pairs:
                continue
            seen_pairs.add(pair)
            lines.append(f"  {node_id(src)} -->|{rel}| {node_id(target)}")

    lines.append("")

    # Clickable links: internal nodes jump to their section anchor.
    for e in sorted(entities, key=kind_sort_key):
        nid = node_id(e["id"])
        anchor = e["id"]
        lines.append(f'  click {nid} "#{anchor}"')

    lines.append("")

    # Styling per kind.
    lines += [
        "  classDef crate fill:#1f6feb,stroke:#0b3d91,color:#fff;",
        "  classDef module fill:#238636,stroke:#0f5323,color:#fff;",
        "  classDef concept fill:#8957e5,stroke:#4b2a8a,color:#fff;",
        "  classDef artifact fill:#bf8700,stroke:#7a5600,color:#fff;",
        "  classDef config fill:#6e7681,stroke:#3a3f47,color:#fff;",
        "  classDef meta fill:#cf222e,stroke:#82071e,color:#fff;",
        "  classDef external fill:#eaeef2,stroke:#8c959f,color:#24292f;",
        "```",
    ]
    return "\n".join(lines)


def build_sections(entities: list[dict]) -> str:
    by_id = {e["id"]: e for e in entities}
    out = ["## Entities", ""]
    for e in sorted(entities, key=kind_sort_key):
        out.append(f"### {e['id']}")
        out.append("")
        out.append(f"**Kind:** {e['kind']}  ")
        out.append("")
        if e["summary"]:
            out.append(e["summary"])
            out.append("")
        if e["body"]:
            out.append(e["body"])
            out.append("")
        if e["related"]:
            out.append("**Related:**")
            out.append("")
            for target, rel in e["related"]:
                if target in by_id:
                    out.append(f"- {rel} → [{target}](#{target})")
                else:
                    out.append(f"- {rel} → `{target}` *(external)*")
            out.append("")
        out.append(f"*Source: [`.context/{e['_file']}`]"
                   f"(https://github.com/yarrib/dex/blob/main/.context/{e['_file']})*")
        out.append("")
    return "\n".join(out)


def main() -> int:
    if not CONTEXT_DIR.is_dir():
        print(f"error: {CONTEXT_DIR} not found", file=sys.stderr)
        return 1

    entities = load_entities()
    if not entities:
        print(f"error: no entity files in {CONTEXT_DIR}", file=sys.stderr)
        return 1

    ids = {e["id"] for e in entities}
    # Warn (don't fail) on dangling internal-looking edges.
    for e in entities:
        for target, _rel in e["related"]:
            if target not in ids and "-" in target and target.islower():
                print(f"warning: {e['_file']} references unknown entity "
                      f"'{target}' (rendered as external)", file=sys.stderr)

    header = [
        "# Repo Knowledge Graph",
        "",
        "> **Generated file — do not edit by hand.** "
        "Built from the `.context/` entity files by "
        "`scripts/gen_context_graph.py`. Edit those files and re-run the "
        "generator to update this page.",
        "",
        f"This graph captures **{len(entities)} entities** across the dex "
        "codebase and how they relate. Click any node to jump to its summary.",
        "",
    ]

    page = "\n".join(header) + "\n" + build_mermaid(entities) + "\n\n" + \
        build_sections(entities)

    OUTPUT.write_text(page.rstrip() + "\n", encoding="utf-8")
    print(f"wrote {OUTPUT.relative_to(REPO_ROOT)} ({len(entities)} entities)")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
