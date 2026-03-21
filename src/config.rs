use crossterm::style::Color;
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
    theme_raw: ThemeRaw,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            max_visible: 10,
            keybindings: KeybindingsRaw::default(),
            theme_raw: ThemeRaw::default(),
        }
    }
}

#[derive(Debug, Deserialize, Default)]
struct ConfigFile {
    #[serde(default)]
    keybindings: KeybindingsRaw,
    #[serde(default)]
    theme: ThemeRaw,
}

#[derive(Debug, Deserialize, Default)]
pub struct KeybindingsRaw {
    pub tab: Option<String>,
    #[serde(rename = "shift-tab")]
    pub shift_tab: Option<String>,
    pub enter: Option<String>,
    pub space: Option<String>,
}

#[derive(Debug, Deserialize, Default)]
struct ThemeRaw {
    border: Option<String>,
    #[serde(rename = "selected-fg")]
    selected_fg: Option<String>,
    #[serde(rename = "selected-bg")]
    selected_bg: Option<String>,
    description: Option<String>,
    filter: Option<String>,
    candidate: Option<String>,
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

pub(crate) fn parse_color(s: &str) -> Option<Color> {
    let s = s.to_lowercase();
    match s.as_str() {
        "black" => Some(Color::Black),
        "dark-red" => Some(Color::DarkRed),
        "red" => Some(Color::Red),
        "dark-green" => Some(Color::DarkGreen),
        "green" => Some(Color::Green),
        "dark-yellow" => Some(Color::DarkYellow),
        "yellow" => Some(Color::Yellow),
        "dark-blue" => Some(Color::DarkBlue),
        "blue" => Some(Color::Blue),
        "dark-magenta" => Some(Color::DarkMagenta),
        "magenta" => Some(Color::Magenta),
        "dark-cyan" => Some(Color::DarkCyan),
        "cyan" => Some(Color::Cyan),
        "dark-grey" | "dark-gray" => Some(Color::DarkGrey),
        "grey" | "gray" => Some(Color::Grey),
        "white" => Some(Color::White),
        _ => {
            if let Some(n) = s.strip_prefix("ansi:") {
                n.parse::<u8>().ok().map(Color::AnsiValue)
            } else if let Some(rgb) = s.strip_prefix("rgb:") {
                let parts: Vec<&str> = rgb.split(',').collect();
                if parts.len() == 3
                    && let (Ok(r), Ok(g), Ok(b)) = (
                        parts[0].parse::<u8>(),
                        parts[1].parse::<u8>(),
                        parts[2].parse::<u8>(),
                    )
                {
                    return Some(Color::Rgb { r, g, b });
                }
                None
            } else {
                None
            }
        }
    }
}

pub struct Theme {
    pub border: Option<Color>,
    pub selected_fg: Option<Color>,
    pub selected_bg: Option<Color>,
    pub description: Color,
    pub filter: Option<Color>,
    pub candidate: Option<Color>,
}

impl Default for Theme {
    fn default() -> Self {
        Theme {
            border: None,
            selected_fg: None,
            selected_bg: None,
            description: Color::DarkGrey,
            filter: None,
            candidate: None,
        }
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
            theme_raw: file.theme,
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

    pub fn theme(&self) -> Theme {
        Theme {
            border: self.theme_raw.border.as_deref().and_then(parse_color),
            selected_fg: self.theme_raw.selected_fg.as_deref().and_then(parse_color),
            selected_bg: self.theme_raw.selected_bg.as_deref().and_then(parse_color),
            description: self
                .theme_raw
                .description
                .as_deref()
                .and_then(parse_color)
                .unwrap_or(Color::DarkGrey),
            filter: self.theme_raw.filter.as_deref().and_then(parse_color),
            candidate: self.theme_raw.candidate.as_deref().and_then(parse_color),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::input::Action;

    // --- parse_action ---

    #[test]
    fn parse_action_move_down() {
        assert_eq!(parse_action("move-down", Action::None), Action::MoveDown);
    }

    #[test]
    fn parse_action_move_up() {
        assert_eq!(parse_action("move-up", Action::None), Action::MoveUp);
    }

    #[test]
    fn parse_action_confirm() {
        assert_eq!(parse_action("confirm", Action::None), Action::Confirm);
    }

    #[test]
    fn parse_action_dismiss() {
        assert_eq!(
            parse_action("dismiss", Action::None),
            Action::DismissWithSpace
        );
    }

    #[test]
    fn parse_action_cancel() {
        assert_eq!(parse_action("cancel", Action::None), Action::Cancel);
    }

    #[test]
    fn parse_action_page_down() {
        assert_eq!(parse_action("page-down", Action::None), Action::PageDown);
    }

    #[test]
    fn parse_action_page_up() {
        assert_eq!(parse_action("page-up", Action::None), Action::PageUp);
    }

    #[test]
    fn parse_action_unknown_returns_default() {
        assert_eq!(parse_action("unknown", Action::MoveDown), Action::MoveDown);
        assert_eq!(parse_action("", Action::Cancel), Action::Cancel);
    }

    // --- key_bindings ---

    #[test]
    fn default_keybindings() {
        let bindings = Config::default().key_bindings();
        assert_eq!(bindings.tab, Action::MoveDown);
        assert_eq!(bindings.shift_tab, Action::MoveUp);
        assert_eq!(bindings.enter, Action::Confirm);
        assert_eq!(bindings.space, Action::DismissWithSpace);
    }

    #[test]
    fn key_bindings_overrides() {
        let config = Config {
            max_visible: 10,
            keybindings: KeybindingsRaw {
                tab: Some("confirm".to_string()),
                shift_tab: None,
                enter: None,
                space: Some("cancel".to_string()),
            },
            theme_raw: ThemeRaw::default(),
        };
        let bindings = config.key_bindings();
        assert_eq!(bindings.tab, Action::Confirm);
        assert_eq!(bindings.shift_tab, Action::MoveUp);
        assert_eq!(bindings.enter, Action::Confirm);
        assert_eq!(bindings.space, Action::Cancel);
    }

    // --- parse_color ---

    #[test]
    fn parse_color_ansi_names() {
        assert_eq!(parse_color("black"), Some(Color::Black));
        assert_eq!(parse_color("red"), Some(Color::Red));
        assert_eq!(parse_color("green"), Some(Color::Green));
        assert_eq!(parse_color("yellow"), Some(Color::Yellow));
        assert_eq!(parse_color("blue"), Some(Color::Blue));
        assert_eq!(parse_color("magenta"), Some(Color::Magenta));
        assert_eq!(parse_color("cyan"), Some(Color::Cyan));
        assert_eq!(parse_color("white"), Some(Color::White));
        assert_eq!(parse_color("dark-red"), Some(Color::DarkRed));
        assert_eq!(parse_color("dark-green"), Some(Color::DarkGreen));
        assert_eq!(parse_color("dark-yellow"), Some(Color::DarkYellow));
        assert_eq!(parse_color("dark-blue"), Some(Color::DarkBlue));
        assert_eq!(parse_color("dark-magenta"), Some(Color::DarkMagenta));
        assert_eq!(parse_color("dark-cyan"), Some(Color::DarkCyan));
        assert_eq!(parse_color("dark-grey"), Some(Color::DarkGrey));
        assert_eq!(parse_color("dark-gray"), Some(Color::DarkGrey));
        assert_eq!(parse_color("grey"), Some(Color::Grey));
        assert_eq!(parse_color("gray"), Some(Color::Grey));
    }

    #[test]
    fn parse_color_case_insensitive() {
        assert_eq!(parse_color("Blue"), Some(Color::Blue));
        assert_eq!(parse_color("CYAN"), Some(Color::Cyan));
        assert_eq!(parse_color("Dark-Grey"), Some(Color::DarkGrey));
    }

    #[test]
    fn parse_color_ansi_number() {
        assert_eq!(parse_color("ansi:0"), Some(Color::AnsiValue(0)));
        assert_eq!(parse_color("ansi:255"), Some(Color::AnsiValue(255)));
    }

    #[test]
    fn parse_color_rgb() {
        assert_eq!(
            parse_color("rgb:255,128,0"),
            Some(Color::Rgb {
                r: 255,
                g: 128,
                b: 0
            })
        );
    }

    #[test]
    fn parse_color_invalid() {
        assert_eq!(parse_color("unknown"), None);
        assert_eq!(parse_color("ansi:256"), None);
        assert_eq!(parse_color("rgb:1,2"), None);
        assert_eq!(parse_color("rgb:a,b,c"), None);
    }

    // --- theme ---

    #[test]
    fn theme_defaults() {
        let theme = Config::default().theme();
        assert_eq!(theme.border, None);
        assert_eq!(theme.selected_fg, None);
        assert_eq!(theme.selected_bg, None);
        assert_eq!(theme.description, Color::DarkGrey);
        assert_eq!(theme.filter, None);
        assert_eq!(theme.candidate, None);
    }

    #[test]
    fn theme_overrides() {
        let config = Config {
            max_visible: 10,
            keybindings: KeybindingsRaw::default(),
            theme_raw: ThemeRaw {
                border: Some("blue".to_string()),
                selected_fg: Some("black".to_string()),
                selected_bg: Some("cyan".to_string()),
                description: Some("dark-grey".to_string()),
                filter: Some("green".to_string()),
                candidate: Some("white".to_string()),
            },
        };
        let theme = config.theme();
        assert_eq!(theme.border, Some(Color::Blue));
        assert_eq!(theme.selected_fg, Some(Color::Black));
        assert_eq!(theme.selected_bg, Some(Color::Cyan));
        assert_eq!(theme.description, Color::DarkGrey);
        assert_eq!(theme.filter, Some(Color::Green));
        assert_eq!(theme.candidate, Some(Color::White));
    }

    #[test]
    fn theme_invalid_falls_back() {
        let config = Config {
            max_visible: 10,
            keybindings: KeybindingsRaw::default(),
            theme_raw: ThemeRaw {
                border: Some("invalid".to_string()),
                description: Some("also-invalid".to_string()),
                ..ThemeRaw::default()
            },
        };
        let theme = config.theme();
        assert_eq!(theme.border, None);
        assert_eq!(theme.description, Color::DarkGrey);
    }
}
