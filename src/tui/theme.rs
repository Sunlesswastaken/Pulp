use std::sync::RwLock;
use std::sync::atomic::{AtomicUsize, Ordering};

use ratatui::style::{Color, Modifier, Style};
use serde_json::Value;

/// Opencode's dark-side theme resolution. Themes ship as opencode-flavoured
/// JSON (`packages/tui/src/theme/assets/*.json`): each file has a `defs`
/// table mapping names → hex and a `theme` table whose values are either a
/// hex / def-hint or a `{ "dark": …, "light": … }` pair. pulp renders a dark
/// UI, so we resolve the `dark` side of every slot.
///
/// Runtime model mirrors opencode: the registry is built once at startup
/// (bundled themes + anything in `~/.config/pulp/themes/*.json`), the active
/// theme is just an index into that list, and switching is O(1).
///
/// Every semantic slot pulp can draw. Values are the `dark` resolutions of
/// the opencode JSON keys of the same camelCase name.
/// the opencode JSON keys of the same camelCase name.
#[derive(Debug, Clone)]
pub struct Theme {
    pub name: String,
    pub text: Color,
    pub text_muted: Color,
    pub primary: Color,
    pub accent: Color,
    pub error: Color,
    pub warning: Color,
    pub success: Color,
    pub info: Color,
    pub background: Color,
    pub background_panel: Color,
    pub background_element: Color,
    pub border: Color,
    pub border_active: Color,
    pub border_subtle: Color,
}

// ---------------------------------------------------------------------------
// Registry + active theme. Thread-safe: rendering reads under a short lock;
// switching swaps the atomic index.
// ---------------------------------------------------------------------------

/// Full set of known themes. Empty until [`init`] runs — in that case
/// [`active_theme`] falls back to the bundled opencode default, which keeps
/// unit tests (which never call [`init`]) deterministic.
static REGISTRY: RwLock<Vec<Theme>> = RwLock::new(Vec::new());

/// Index into [`REGISTRY`] of the theme currently being rendered.
static ACTIVE: AtomicUsize = AtomicUsize::new(0);

/// Whether [`init`] has already run. Lets `init` be called more than once
/// without rebuilding or re-selecting the registry.
static READY: AtomicUsize = AtomicUsize::new(0);

fn active_theme() -> Theme {
    let reg = REGISTRY.read().unwrap();
    if reg.is_empty() {
        Theme::opencode()
    } else {
        reg[ACTIVE.load(Ordering::Relaxed).min(reg.len() - 1)].clone()
    }
}

/// Look up a registered theme by name (used by the picker for swatches).
pub fn get(name: &str) -> Option<Theme> {
    REGISTRY
        .read()
        .unwrap()
        .iter()
        .find(|t| t.name == name)
        .cloned()
}

/// Activate a theme by name, persisting the choice to the config file.
/// Returns `false` if the name isn't registered.
pub fn set_active(name: &str) -> bool {
    let reg = REGISTRY.read().unwrap();
    let Some(index) = reg.iter().position(|t| t.name == name) else {
        return false;
    };
    ACTIVE.store(index, Ordering::Relaxed);
    drop(reg);
    persist_choice(name);
    true
}

/// Names of every registered theme, for the in-app picker and `--theme`.
pub fn names() -> Vec<String> {
    let reg = REGISTRY.read().unwrap();
    if reg.is_empty() {
        vec!["opencode".to_string()]
    } else {
        reg.iter().map(|t| t.name.clone()).collect()
    }
}

// ---------------------------------------------------------------------------
// Bootstrap — bundled themes + user themes + config
// ---------------------------------------------------------------------------

/// Bundled themes, embedded as opencode JSON at compile time.
const BUNDLED: &[(&str, &str)] = &[
    ("opencode", include_str!("themes/opencode.json")),
    ("dracula", include_str!("themes/dracula.json")),
    ("everforest", include_str!("themes/everforest.json")),
    ("flexoki", include_str!("themes/flexoki.json")),
    ("gruvbox", include_str!("themes/gruvbox.json")),
    ("kanagawa", include_str!("themes/kanagawa.json")),
    ("monokai", include_str!("themes/monokai.json")),
    ("nord", include_str!("themes/nord.json")),
    ("one-dark", include_str!("themes/one-dark.json")),
    ("palenight", include_str!("themes/palenight.json")),
    ("rosepine", include_str!("themes/rosepine.json")),
    ("solarized", include_str!("themes/solarized.json")),
    ("tokyonight", include_str!("themes/tokyonight.json")),
];

/// OpenCode honours an explicit `--theme` (their `--theme` flag) over the
/// config file's `theme` key; we mirror that. Loads the bundled themes plus
/// any user-supplied JSON themes from `~/.config/pulp/themes/*.json`, then
/// activates the best preference.
pub fn init(forced_theme: Option<&str>) {
    if READY.swap(1, Ordering::Relaxed) == 1 {
        return;
    }

    let mut themes = vec![Theme::opencode()];
    for (name, raw) in BUNDLED {
        if let Some(theme) = Theme::from_json(name, raw) {
            themes.push(theme);
        }
    }
    if let Some(dir) = config_dir() {
        let themes_dir = dir.join("themes");
        let Ok(entries) = std::fs::read_dir(&themes_dir) else {
            if forced_theme.is_some() {
                eprintln!("warning: theme dir {themes_dir:?} is unreadable");
            }
            return finish_init(themes, forced_theme);
        };
        let mut files: Vec<_> = entries
            .filter_map(|e| e.ok())
            .filter(|e| e.path().extension().and_then(|e| e.to_str()) == Some("json"))
            .collect();
        files.sort_by_key(|e| e.file_name());
        for entry in files {
            let path = entry.path();
            let Ok(raw) = std::fs::read_to_string(&path) else {
                continue;
            };
            let name = path
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("theme")
                .to_string();
            if let Some(theme) = Theme::from_json(&name, &raw) {
                themes.push(theme);
            }
        }
    }
    finish_init(themes, forced_theme);
}

fn finish_init(themes: Vec<Theme>, forced_theme: Option<&str>) {
    // Prefer an explicit --theme, then the config file's theme key.
    let config_theme = config_dir()
        .and_then(|dir| std::fs::read_to_string(dir.join("config.json")).ok())
        .and_then(|raw| serde_json::from_str::<Value>(&raw).ok())
        .and_then(|v| v.get("theme").and_then(|t| t.as_str()).map(String::from));

    let preferred = forced_theme
        .or(config_theme.as_deref())
        .unwrap_or("opencode");

    if let Some(index) = themes.iter().position(|t| t.name == preferred) {
        ACTIVE.store(index, Ordering::Relaxed);
    } else if forced_theme.is_some() {
        eprintln!("warning: unknown theme `{preferred}`, using `opencode`");
    }

    *REGISTRY.write().unwrap() = themes;
}

/// Config directory: `$PULP_CONFIG_DIR` (test override), `$XDG_CONFIG_HOME/pulp`,
/// else `~/.config/pulp`. Matches opencode's `~/.config/opencode` layout.
#[cfg(test)]
pub fn config_dir() -> Option<std::path::PathBuf> {
    TEST_CONFIG_DIR.read().unwrap().clone()
}

#[cfg(test)]
static TEST_CONFIG_DIR: RwLock<Option<std::path::PathBuf>> = RwLock::new(None);

#[cfg(test)]
pub fn set_test_config_dir(dir: std::path::PathBuf) {
    *TEST_CONFIG_DIR.write().unwrap() = Some(dir);
}

#[cfg(not(test))]
pub fn config_dir() -> Option<std::path::PathBuf> {
    if let Ok(dir) = std::env::var("PULP_CONFIG_DIR") {
        return Some(std::path::PathBuf::from(dir));
    }
    let base = std::env::var("XDG_CONFIG_HOME")
        .ok()
        .and_then(|d| (!d.is_empty()).then(|| std::path::PathBuf::from(d)))
        .or_else(|| {
            std::env::var("HOME")
                .ok()
                .map(|h| std::path::PathBuf::from(h).join(".config"))
        })?;
    Some(base.join("pulp"))
}

/// Persist the chosen theme to the config file: `{"theme": "<name>"}`.
/// Best-effort — missing config dir just means the choice isn't saved.
fn persist_choice(name: &str) {
    let Some(dir) = config_dir() else { return };
    let Ok(_) = std::fs::create_dir_all(&dir) else {
        return;
    };
    let json = serde_json::json!({ "theme": name });
    let _ = std::fs::write(
        dir.join("config.json"),
        serde_json::to_vec_pretty(&json).unwrap(),
    );
}

// ---------------------------------------------------------------------------
// Theme parsing (opencode JSON format)
// ---------------------------------------------------------------------------

impl Theme {
    fn opencode() -> Theme {
        Theme {
            name: "opencode".to_string(),
            text: Color::Rgb(0xee, 0xee, 0xee),
            text_muted: Color::Rgb(0x80, 0x80, 0x80),
            primary: Color::Rgb(0xfa, 0xb2, 0x83),
            accent: Color::Rgb(0x9d, 0x7c, 0xd8),
            error: Color::Rgb(0xe0, 0x6c, 0x75),
            warning: Color::Rgb(0xf5, 0xa7, 0x42),
            success: Color::Rgb(0x7f, 0xd8, 0x8f),
            info: Color::Rgb(0x56, 0xb6, 0xc2),
            background: Color::Rgb(0x0a, 0x0a, 0x0a),
            background_panel: Color::Rgb(0x14, 0x14, 0x14),
            background_element: Color::Rgb(0x1e, 0x1e, 0x1e),
            border: Color::Rgb(0x48, 0x48, 0x48),
            border_active: Color::Rgb(0x60, 0x60, 0x60),
            border_subtle: Color::Rgb(0x3c, 0x3c, 0x3c),
        }
    }

    /// Parse an opencode theme file, resolving the `dark` value of every slot
    /// through the file's `defs` table. Missing/invalid slots fall back to the
    /// opencode default so a theme can never nuke the UI completely.
    fn from_json(name: &str, raw: &str) -> Option<Theme> {
        let root: Value = serde_json::from_str(raw).ok()?;
        let theme_obj = root.get("theme")?;
        let defs = root.get("defs").and_then(|d| d.as_object())?;

        let store: Theme = Theme {
            name: name.to_string(),
            text: slot(theme_obj, defs, "text"),
            text_muted: slot(theme_obj, defs, "textMuted"),
            primary: slot(theme_obj, defs, "primary"),
            accent: slot(theme_obj, defs, "accent"),
            error: slot(theme_obj, defs, "error"),
            warning: slot(theme_obj, defs, "warning"),
            success: slot(theme_obj, defs, "success"),
            info: slot(theme_obj, defs, "info"),
            background: slot(theme_obj, defs, "background"),
            background_panel: slot(theme_obj, defs, "backgroundPanel"),
            background_element: slot(theme_obj, defs, "backgroundElement"),
            border: slot(theme_obj, defs, "border"),
            border_active: slot(theme_obj, defs, "borderActive"),
            border_subtle: slot(theme_obj, defs, "borderSubtle"),
        };

        // Fill slots the theme doesn't define with opencode defaults.
        let base = Theme::opencode();
        let fill = |own: Option<Color>, fallback: Color| own.unwrap_or(fallback);
        Some(Theme {
            name: store.name,
            text: fill(Some(store.text).filter(|c| *c != Color::Reset), base.text),
            text_muted: fill(
                Some(store.text_muted).filter(|c| *c != Color::Reset),
                base.text_muted,
            ),
            primary: fill(
                Some(store.primary).filter(|c| *c != Color::Reset),
                base.primary,
            ),
            accent: fill(
                Some(store.accent).filter(|c| *c != Color::Reset),
                base.accent,
            ),
            error: fill(Some(store.error).filter(|c| *c != Color::Reset), base.error),
            warning: fill(
                Some(store.warning).filter(|c| *c != Color::Reset),
                base.warning,
            ),
            success: fill(
                Some(store.success).filter(|c| *c != Color::Reset),
                base.success,
            ),
            info: fill(Some(store.info).filter(|c| *c != Color::Reset), base.info),
            background: fill(
                Some(store.background).filter(|c| *c != Color::Reset),
                base.background,
            ),
            background_panel: fill(
                Some(store.background_panel).filter(|c| *c != Color::Reset),
                base.background_panel,
            ),
            background_element: fill(
                Some(store.background_element).filter(|c| *c != Color::Reset),
                base.background_element,
            ),
            border: fill(
                Some(store.border).filter(|c| *c != Color::Reset),
                base.border,
            ),
            border_active: fill(
                Some(store.border_active).filter(|c| *c != Color::Reset),
                base.border_active,
            ),
            border_subtle: fill(
                Some(store.border_subtle).filter(|c| *c != Color::Reset),
                base.border_subtle,
            ),
        })
    }
}

/// Resolve one slot: the theme value (hex / def name / {dark,light}) via
/// `defs`, favouring opencode's default when the value is `Color::Reset`.
fn slot(theme_obj: &Value, defs: &serde_json::Map<String, Value>, key: &str) -> Color {
    let value = theme_obj
        .get(key)
        .and_then(|v| resolve_dark(defs, v))
        .or_else(|| defs.get(key).and_then(|d| resolve_dark(defs, d)))
        .unwrap_or_default();
    if value == Color::Reset {
        default_slot(key)
    } else {
        value
    }
}

fn default_slot(key: &str) -> Color {
    match key {
        "text" => Color::Rgb(0xee, 0xee, 0xee),
        "textMuted" => Color::Rgb(0x80, 0x80, 0x80),
        "primary" => Color::Rgb(0xfa, 0xb2, 0x83),
        "accent" => Color::Rgb(0x9d, 0x7c, 0xd8),
        "error" => Color::Rgb(0xe0, 0x6c, 0x75),
        "warning" => Color::Rgb(0xf5, 0xa7, 0x42),
        "success" => Color::Rgb(0x7f, 0xd8, 0x8f),
        "info" => Color::Rgb(0x56, 0xb6, 0xc2),
        "background" => Color::Rgb(0x0a, 0x0a, 0x0a),
        "backgroundPanel" => Color::Rgb(0x14, 0x14, 0x14),
        "backgroundElement" => Color::Rgb(0x1e, 0x1e, 0x1e),
        "border" => Color::Rgb(0x48, 0x48, 0x48),
        "borderActive" => Color::Rgb(0x60, 0x60, 0x60),
        "borderSubtle" => Color::Rgb(0x3c, 0x3c, 0x3c),
        _ => Color::Reset,
    }
}

/// Resolve one color value: either a `#rrggbb` hex string, a name from the
/// file's `defs`, or a `{ "dark": …, "light": … }` object (dark side taken).
fn resolve_dark(defs: &serde_json::Map<String, Value>, value: &Value) -> Option<Color> {
    match value {
        Value::String(s) => {
            if let Some(hex) = s.strip_prefix('#') {
                hex_to_color(hex)
            } else {
                defs.get(s).and_then(|def| resolve_dark(defs, def))
            }
        }
        Value::Object(obj) => obj.get("dark").and_then(|dark| resolve_dark(defs, dark)),
        _ => None,
    }
}

fn hex_to_color(hex: &str) -> Option<Color> {
    if hex.len() != 6 {
        return None;
    }
    let r = u8::from_str_radix(&hex[0..2], 16).ok()?;
    let g = u8::from_str_radix(&hex[2..4], 16).ok()?;
    let b = u8::from_str_radix(&hex[4..6], 16).ok()?;
    Some(Color::Rgb(r, g, b))
}

// ---------------------------------------------------------------------------
// Convenience styles — read the active theme
// ---------------------------------------------------------------------------

/// Accent (violet) — highlights, selection marker, tip bullet.
pub fn accent() -> Style {
    Style::new().fg(active_theme().accent)
}

/// Muted (textMuted) — hints, descriptions, borders.
pub fn muted() -> Style {
    Style::new().fg(active_theme().text_muted)
}

pub fn muted_bold() -> Style {
    muted().add_modifier(Modifier::BOLD)
}

pub fn muted_italic() -> Style {
    muted().add_modifier(Modifier::ITALIC)
}

/// Header text (text) — logo right half, breadcrumbs, bold hints.
pub fn header() -> Style {
    Style::new()
        .fg(active_theme().text)
        .add_modifier(Modifier::BOLD)
}

pub fn header_color() -> Color {
    active_theme().text
}

pub fn error() -> Style {
    Style::new().fg(active_theme().error)
}

/// Accent + bold, used by the breadcrumb brand.
pub fn accent_bold() -> Style {
    accent().add_modifier(Modifier::BOLD)
}

/// Background element style — subtle fill behind the prompt box.
pub fn bg_element() -> Style {
    Style::new().bg(active_theme().background_element)
}

// ---------------------------------------------------------------------------
// Tint / shadow — used by the header mark system
// ---------------------------------------------------------------------------

/// OpenCode's `tint` — blend `overlay` toward `base` by `alpha` (0..=1).
pub fn tint(base: Color, overlay: Color, alpha: f32) -> Color {
    let mix = |b: u8, o: u8| (b as f32 + (o as f32 - b as f32) * alpha).round() as u8;
    match (base, overlay) {
        (Color::Rgb(br, bg, bb), Color::Rgb(or, og, ob)) => {
            Color::Rgb(mix(br, or), mix(bg, og), mix(bb, ob))
        }
        _ => overlay,
    }
}

/// Shadow tone for header glyph cells, mirroring `Logo()`:
/// `shadow = tint(theme.background, fg, 0.25)`.
pub fn shadow(fg: Color) -> Color {
    tint(active_theme().background, fg, 0.25)
}

// ---------------------------------------------------------------------------
// Semantic slot accessors (active theme)
// ---------------------------------------------------------------------------

/// Active `background` (page) colour.
pub fn page_bg() -> Color {
    active_theme().background
}

/// Active `text` colour.
pub fn text() -> Color {
    active_theme().text
}

/// Active `textMuted` colour.
pub fn text_muted() -> Color {
    active_theme().text_muted
}

/// Active `primary` colour (peach in opencode's defaults).
#[allow(dead_code)] // thematic future use (progress/primary-action accents)
pub fn primary() -> Color {
    active_theme().primary
}

/// Active `accent` colour (highlights, tip, prompt left strip).
pub fn accent_color() -> Color {
    active_theme().accent
}

/// Active `warning` colour (tip markers).
pub fn warning() -> Color {
    active_theme().warning
}

/// Active `info` colour (status info).
pub fn info() -> Color {
    active_theme().info
}

/// Active `success` colour (status success).
pub fn success() -> Color {
    active_theme().success
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_opencode_theme_hex_and_defs() {
        let json = r##"{
            "defs": {
                "bg": "#101020",
                "fg": "#fff0e0"
            },
            "theme": {
                "text": "fg",
                "background": "#101020",
                "accent": { "dark": "fg", "light": "#000" },
                "backgroundPanel": { "dark": "bg" }
            }
        }"##;
        let theme = Theme::from_json("peach", json).unwrap();
        assert_eq!(theme.name, "peach");
        assert_eq!(theme.text, Color::Rgb(0xff, 0xf0, 0xe0));
        assert_eq!(theme.background, Color::Rgb(0x10, 0x10, 0x20));
        assert_eq!(theme.background_panel, Color::Rgb(0x10, 0x10, 0x20));
        // Undefined slots fall back to opencode defaults rather than breaking.
        assert_eq!(theme.border, Color::Rgb(0x48, 0x48, 0x48));
    }

    #[test]
    fn hex_to_color_rejects_bad_input() {
        assert!(hex_to_color("ff0000").is_some());
        assert!(hex_to_color("f00").is_none());
        assert!(hex_to_color("zzz000").is_none());
    }

    #[test]
    fn every_bundled_theme_parses() {
        for (name, raw) in BUNDLED {
            let theme = Theme::from_json(name, raw)
                .unwrap_or_else(|| panic!("bundled theme `{name}` failed to parse"));
            assert!(!theme.name.is_empty());
            assert_ne!(theme.text, Color::Reset, "`{name}` missing text");
            assert_ne!(
                theme.background,
                Color::Reset,
                "`{name}` missing background"
            );
            assert_ne!(theme.accent, Color::Reset, "`{name}` missing accent");
        }
    }

    #[test]
    fn bundled_themes_resolve_distinct_accents() {
        // A smoke toggle: loading all themes should give named, non-default slots.
        let opencode = Theme::opencode();
        for (name, raw) in BUNDLED {
            let theme = Theme::from_json(name, raw)
                .unwrap_or_else(|| panic!("bundled theme `{name}` failed to parse"));
            assert!(
                theme.accent != opencode.accent || *name == "opencode",
                "`{name}` accent looks like the default — defs resolution may be flat"
            );
        }
    }

    #[test]
    fn init_loads_user_themes_honours_config_and_persists_choice() {
        // Point the config dir at a throwaway temp dir with a user theme and
        // a config file that asks for it by name.
        let dir = std::env::temp_dir().join(format!("pulp-theme-test-{}", std::process::id()));
        let themes_dir = dir.join("themes");
        std::fs::create_dir_all(&themes_dir).unwrap();
        std::fs::write(
            themes_dir.join("midnight.json"),
            r##"{
                "defs": {
                    "ink": "#0b0b1f",
                    "paper": "#d8d8ff",
                    "grape": "#8b5cf6"
                },
                "theme": {
                    "text": "paper",
                    "textMuted": { "dark": "paper", "light": "ink" },
                    "accent": "grape",
                    "primary": "#ffd580",
                    "background": "ink",
                    "border": { "dark": "grape", "light": "ink" }
                }
            }"##,
        )
        .unwrap();
        std::fs::write(dir.join("config.json"), r##"{ "theme": "midnight" }"##).unwrap();

        set_test_config_dir(dir.clone());
        init(None);

        // User theme is registered and its config-file preference is active.
        assert!(
            names().contains(&"midnight".to_string()),
            "user theme `midnight` should be registered, got {:?}",
            names()
        );
        let midnight = get("midnight").expect("midnight registered");
        assert_eq!(midnight.accent, Color::Rgb(0x8b, 0x5c, 0xf6));
        assert_eq!(
            text_muted(),
            midnight.text_muted,
            "active theme should be midnight"
        );

        // Switching persists the new choice to the config file.
        assert!(
            set_active("nord"),
            "nord is bundled — set_active should succeed"
        );
        let persisted: Value =
            serde_json::from_str(&std::fs::read_to_string(dir.join("config.json")).unwrap())
                .unwrap();
        assert_eq!(persisted["theme"], "nord");

        // Unknown themes are rejected without touching the active theme.
        set_active("midnight");
        assert!(!set_active("does-not-exist"));
        assert_eq!(
            text_muted(),
            get("midnight").unwrap().text_muted,
            "active theme unchanged after failed set_active"
        );

        // Cleanup.
        let _ = std::fs::remove_dir_all(&dir);
        set_test_config_dir(std::path::PathBuf::new());
    }
}
