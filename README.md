# Pulp

A fast, native PDF toolkit for the terminal.

Two interfaces, one tool:

- **TUI** — run `pulp` with no arguments to open the interactive terminal UI
- **CLI** — plain subcommands such as `pulp compress document.pdf`

## Operations

| Operation | CLI |
| --- | --- |
| Compress (headline) | `pulp compress document.pdf` |
| Merge | `pulp merge a.pdf b.pdf -o merged.pdf` |
| Split | `pulp split document.pdf` |
| Remove pages | `pulp remove document.pdf` |
| Extract pages | `pulp extract document.pdf` |
| Password | `pulp password document.pdf` |
| Info | `pulp info document.pdf` |

## Status

Phase 1 (current): CLI shell + TUI shell, architecture established, test
workflow in place. PDF operations land incrementally in later phases.

Compression is backed by Ghostscript (`gs`) rather than a from-scratch
codec; `lopdf` covers in-process parsing and manipulation.

## Architecture

```
src/
├── main.rs         entry point: CLI vs TUI dispatch, top-level error output
├── cli/            clap definitions — the command vocabulary
├── operations/     turns a parsed CLI command into a concrete job
├── core/           shared domain model (the Operation enum)
├── pdf/            lopdf document model (later phase)
├── compression/    compression backends, Ghostscript first (later phase)
├── tui/            event loop, screen states, reusable components, theme
└── error.rs        single typed error enum
```

The CLI and TUI share one domain vocabulary (`core::operation::Operation`),
so the two interfaces cannot drift apart.

## Build & test

```sh
cargo build
cargo test
cargo run -- --help
cargo run           # opens the TUI
```

## TUI design

Restrained by design: terminal-default background, a single muted accent
for selection, no borders or panels. Reusable components (navigation
lists, key-hint footer, status lines, breadcrumb header) keep every
screen consistent.
