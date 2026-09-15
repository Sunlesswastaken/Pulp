# Project memory

Persistent notes for the opencode agent working on pulp (Rust + ratatui TUI for PDF compression/merge/split/remove/extract/password).

## Active TODOs

- **Theming needs a full rework.** The current theme system ("opencode-theme-JSON port, 13 bundled themes, --theme flag, Ctrl+T picker, config persistence") is a first pass and is NOT final. The user wants:
  1. A **system theme** — auto-detect light/dark from the OS/terminal and pick accordingly (the JSON files ship both `dark` and `light` sides but pulp only resolves `dark`; resolve the `light` side and switch based on system preference).
  2. Follow the OS theme / system preference generally (dark ↔ light), not just a hardcoded dark UI.
  3. Overall theming to be "fixed entirely" — treat the whole color/theme pipeline as unfinished.

## Work history highlights

- Committed as `281321a` (feat: theme system with 13 opencode themes, picker, show accent strip on prompt). All prior opencode-UI work (`ec59481`, `1acca2b`) is on main.
- Theme system: `src/tui/theme.rs` — Theme struct (14 color slots), RwLock registry + AtomicUsize active index, opencode JSON parser (defs + dark side only so far), 13 bundled themes via `include_str!`, user themes from `~/.config/pulp/themes/*.json`, config-file preference (`{"theme": "name"}`), `--theme` override, style/color accessors read the active theme.
- Prompt box left strip = `theme::accent_color()`, panel `backgroundElement` fill starts at `x+1` (detached strip, mirrors opencode's `borderHighlight` + `paddingLeft=2`). Tip uses `theme::warning()` for the bullet and `text_muted` for the copy.
- Open unresolved question (unsolved at last session): user reported the prompt strip colour "doesn't match the theme, it's matching the tip text colour". Not reproducible in render tests (strip = accent violet in `opencode`, teal in `nord`); suspected stale binary, needs a rebuild + live inspection next time.
- Rust edition 2024, `#![forbid(unsafe_code)]` in main.rs → no `std::env::set_var`; tests inject config dir via `set_test_config_dir` (RwLock static).
- Config dir: `$PULP_CONFIG_DIR` (prod only) / `$XDG_CONFIG_HOME/pulp` / `~/.config/pulp`.
- Uncommitted leftovers: `examples/scratch.rs`, `benches/.gitkeep`. Push `main` (currently ahead of origin) when user asks.

## Conventions

- `cargo test` = 26 unit + 3 integration (current baseline).
- `cargo clippy --all-targets` must be zero-warning; `cargo fmt --check` clean.
- Do NOT commit/push unless the user explicitly asks.