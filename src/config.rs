use serde::Deserialize;
use std::fs;
use std::path::PathBuf;

use crate::input::Action;

fn config_path() -> Option<PathBuf> {
    dirs::config_dir().map(|d| d.join("zacrs").join("config.toml"))
}

pub struct Config {
    pub max_visible: usize,
    pub keybindings: KeybindingsRaw,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            max_visible: 10,
            keybindings: KeybindingsRaw::default(),
        }
    }
}

#[derive(Debug, Deserialize, Default)]
struct ConfigFile {
    #[serde(default)]
    keybindings: KeybindingsRaw,
}

#[derive(Debug, Deserialize, Default)]
pub struct KeybindingsRaw {
    pub tab: Option<String>,
    #[serde(rename = "shift-tab")]
    pub shift_tab: Option<String>,
    pub enter: Option<String>,
    pub space: Option<String>,
}

/// Parsed keybindings passed to read_action
#[derive(Debug, Clone)]
pub struct KeyBindings {
    pub tab: Action,
    pub shift_tab: Action,
    pub enter: Action,
    pub space: Action,
}

impl Default for KeyBindings {
    fn default() -> Self {
        KeyBindings {
            tab: Action::MoveDown,
            shift_tab: Action::MoveUp,
            enter: Action::Confirm,
            space: Action::DismissWithSpace,
        }
    }
}

fn parse_action(s: &str, default: Action) -> Action {
    match s {
        "move-down" => Action::MoveDown,
        "move-up" => Action::MoveUp,
        "confirm" => Action::Confirm,
        "dismiss" => Action::DismissWithSpace,
        "cancel" => Action::Cancel,
        "page-down" => Action::PageDown,
        "page-up" => Action::PageUp,
        _ => default,
    }
}

impl Config {
    pub fn load() -> Self {
        let Some(path) = config_path() else {
            return Config::default();
        };
        let Ok(content) = fs::read_to_string(&path) else {
            return Config::default();
        };
        let file: ConfigFile = toml::from_str(&content).unwrap_or_default();
        Config {
            max_visible: 10,
            keybindings: file.keybindings,
        }
    }

    pub fn key_bindings(&self) -> KeyBindings {
        let defaults = KeyBindings::default();
        KeyBindings {
            tab: self
                .keybindings
                .tab
                .as_deref()
                .map(|s| parse_action(s, defaults.tab))
                .unwrap_or(defaults.tab),
            shift_tab: self
                .keybindings
                .shift_tab
                .as_deref()
                .map(|s| parse_action(s, defaults.shift_tab))
                .unwrap_or(defaults.shift_tab),
            enter: self
                .keybindings
                .enter
                .as_deref()
                .map(|s| parse_action(s, defaults.enter))
                .unwrap_or(defaults.enter),
            space: self
                .keybindings
                .space
                .as_deref()
                .map(|s| parse_action(s, defaults.space))
                .unwrap_or(defaults.space),
        }
    }
}
