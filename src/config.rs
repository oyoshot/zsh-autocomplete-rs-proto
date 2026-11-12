use crossterm::style::Color;
use globset::{Glob, GlobMatcher};
use serde::Deserialize;
use std::collections::{BTreeMap, HashMap};
use std::env;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::time::SystemTime;

use crate::input::Action;

/// A precomputed rank for each of Candidate::kind_priority's default groups.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct KindOrder {
    priorities: [u8; 8],
}

impl Default for KindOrder {
    fn default() -> Self {
        Self {
            priorities: [0, 1, 2, 3, 4, 5, 6, 7],
        }
    }
}

impl KindOrder {
    /// Listed groups lead; omitted groups follow in their default order.
    pub fn new(kinds: &[&str]) -> Result<Self, String> {
        const KINDS: [&str; 8] = [
            "directory",
            "file",
            "abbreviation",
            "command",
            "alias",
            "builtin",
            "function",
            "other",
        ];
        if kinds.len() > KINDS.len() {
            return Err("completion.kind_order accepts at most eight kinds".into());
        }
        let mut priorities = [u8::MAX; 8];
        for (rank, kind) in kinds.iter().enumerate() {
            let Some(index) = KINDS.iter().position(|known| known == kind) else {
                return Err(format!("unknown completion.kind_order kind: {kind}"));
            };
            if priorities[index] != u8::MAX {
                return Err(format!("duplicate completion.kind_order kind: {kind}"));
            }
            priorities[index] = rank as u8;
        }
        let mut next = kinds.len() as u8;
        for priority in &mut priorities {
            if *priority == u8::MAX {
                *priority = next;
                next += 1;
            }
        }
        Ok(Self { priorities })
    }

    pub(crate) fn priority(&self, default_priority: u8) -> u8 {
        self.priorities[usize::from(default_priority)]
    }
}

impl<'de> Deserialize<'de> for KindOrder {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let kinds = Vec::<String>::deserialize(deserializer)?;
        let names: Vec<_> = kinds.iter().map(String::as_str).collect();
        Self::new(&names).map_err(serde::de::Error::custom)
    }
}

const MAX_COMMAND_GLOB_PATTERNS: usize = 256;
const MAX_COMMAND_GLOB_BYTES: usize = 1024;
const MAX_COMMAND_CONTEXT_BYTES: usize = 16 * 1024;

fn config_path_from(
    xdg_config_home: Option<PathBuf>,
    home_dir: Option<PathBuf>,
) -> Option<PathBuf> {
    xdg_config_home
        .filter(|path| path.is_absolute())
        .or_else(|| home_dir.map(|home| home.join(".config")))
        .map(|dir| dir.join("zacrs").join("config.toml"))
}

pub(crate) fn config_path() -> Option<PathBuf> {
    config_path_from(
        env::var_os("XDG_CONFIG_HOME").map(PathBuf::from),
        dirs::home_dir(),
    )
}

/// Tracks file changes separately from the settings used by a completion session.
pub(crate) struct ConfigReloader {
    path: PathBuf,
    last_attempt: Option<(SystemTime, u64)>,
}

impl ConfigReloader {
    pub(crate) fn new(path: PathBuf) -> Self {
        Self {
            path,
            last_attempt: None,
        }
    }

    pub(crate) fn reload(&mut self) -> io::Result<Option<Config>> {
        let metadata = match fs::metadata(&self.path) {
            Ok(metadata) => metadata,
            Err(error) if error.kind() == io::ErrorKind::NotFound => {
                // Editors may temporarily remove the file while saving. Keep the
                // active settings, but retry if the file reappears unchanged.
                self.last_attempt = None;
                return Ok(None);
            }
            Err(error) => return Err(error),
        };
        let version = (metadata.modified()?, metadata.len());
        if self.last_attempt == Some(version) {
            return Ok(None);
        }

        let result = Config::load_from(&self.path);
        // Do not repeatedly parse/log the same invalid file. I/O failures remain
        // retryable even if the file's modification time has not changed.
        if result.is_ok()
            || result
                .as_ref()
                .is_err_and(|error| error.kind() == io::ErrorKind::InvalidData)
        {
            self.last_attempt = Some(version);
        }
        result.map(Some)
    }
}

#[derive(Clone)]
pub struct Config {
    pub max_visible: usize,
    pub auto_insert_unambiguous: bool,
    pub expand_abbreviation_on_enter: bool,
    pub expand_abbreviation_on_space: bool,
    pub kind_order: KindOrder,
    pub suffixes: SuffixConfig,
    pub keybindings: KeybindingsRaw,
    pub abbreviations: Vec<Abbreviation>,
    theme_raw: ThemeRaw,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            max_visible: 10,
            auto_insert_unambiguous: true,
            expand_abbreviation_on_enter: false,
            expand_abbreviation_on_space: false,
            kind_order: KindOrder::default(),
            suffixes: SuffixConfig::default(),
            keybindings: KeybindingsRaw::default(),
            abbreviations: Vec::new(),
            theme_raw: ThemeRaw::default(),
        }
    }
}

fn default_true() -> bool {
    true
}

// Manual Default impl required: bool::default() is false, but this option defaults to true.
#[derive(Debug, Deserialize)]
struct CompletionRaw {
    #[serde(default = "default_true")]
    auto_insert_unambiguous: bool,
    #[serde(default)]
    expand_abbreviation_on_enter: bool,
    #[serde(default)]
    expand_abbreviation_on_space: bool,
    #[serde(default)]
    kind_order: KindOrder,
}

impl Default for CompletionRaw {
    fn default() -> Self {
        CompletionRaw {
            auto_insert_unambiguous: true,
            expand_abbreviation_on_enter: false,
            expand_abbreviation_on_space: false,
            kind_order: KindOrder::default(),
        }
    }
}

#[derive(Debug, Deserialize, Default)]
struct ConfigFile {
    #[serde(default)]
    keybindings: KeybindingsRaw,
    #[serde(default)]
    theme: ThemeRaw,
    #[serde(default)]
    completion: CompletionRaw,
    #[serde(default)]
    suffix: SuffixRaw,
    #[serde(default)]
    abbreviations: HashMap<String, String>,
    #[serde(default, rename = "abbreviation")]
    abbreviation: Vec<AbbreviationRaw>,
}

#[derive(Debug, Clone)]
pub struct Abbreviation {
    pub trigger: String,
    pub expansion: String,
    pub description: String,
    pub when: AbbreviationWhen,
}

impl PartialEq for Abbreviation {
    fn eq(&self, other: &Self) -> bool {
        self.trigger == other.trigger
            && self.expansion == other.expansion
            && self.description == other.description
            && self.when == other.when
    }
}

impl Eq for Abbreviation {}

#[derive(Debug, Clone, Copy, Default, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum AbbreviationPosition {
    #[default]
    Command,
    Argument,
    Any,
}

#[derive(Debug, Clone, Default)]
pub struct AbbreviationWhen {
    pub position: AbbreviationPosition,
    command_patterns: Vec<String>,
    command_matchers: Vec<GlobMatcher>,
}

impl Abbreviation {
    pub fn is_available_at(&self, command_position: bool, command_context: Option<&str>) -> bool {
        let position_matches = match self.when.position {
            AbbreviationPosition::Command => command_position,
            AbbreviationPosition::Argument => !command_position,
            AbbreviationPosition::Any => true,
        };
        if !position_matches {
            return false;
        }
        self.when.matches_command(command_context)
    }
}

impl PartialEq for AbbreviationWhen {
    fn eq(&self, other: &Self) -> bool {
        self.position == other.position && self.command_patterns == other.command_patterns
    }
}

impl Eq for AbbreviationWhen {}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
enum OneOrMany {
    One(String),
    Many(Vec<String>),
}

#[derive(Debug, Default, Deserialize)]
struct AbbreviationWhenRaw {
    #[serde(default)]
    position: AbbreviationPosition,
    command: Option<OneOrMany>,
}

impl<'de> Deserialize<'de> for AbbreviationWhen {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let raw = AbbreviationWhenRaw::deserialize(deserializer)?;
        let command_was_set = raw.command.is_some();
        let command_patterns = match raw.command {
            None => Vec::new(),
            Some(OneOrMany::One(pattern)) => vec![pattern],
            Some(OneOrMany::Many(patterns)) => patterns,
        };
        if command_patterns.is_empty() && command_was_set {
            return Err(serde::de::Error::custom(
                "when.command must contain at least one glob",
            ));
        }
        if command_patterns.len() > MAX_COMMAND_GLOB_PATTERNS {
            return Err(serde::de::Error::custom(format!(
                "when.command accepts at most {MAX_COMMAND_GLOB_PATTERNS} globs"
            )));
        }
        if let Some(pattern) = command_patterns
            .iter()
            .find(|pattern| pattern.len() > MAX_COMMAND_GLOB_BYTES)
        {
            return Err(serde::de::Error::custom(format!(
                "when.command glob exceeds {MAX_COMMAND_GLOB_BYTES} bytes: {pattern}"
            )));
        }
        let command_matchers = command_patterns
            .iter()
            .map(|pattern| {
                Glob::new(pattern)
                    .map(|glob| glob.compile_matcher())
                    .map_err(serde::de::Error::custom)
            })
            .collect::<Result<_, _>>()?;
        Ok(Self {
            position: raw.position,
            command_patterns,
            command_matchers,
        })
    }
}

impl AbbreviationWhen {
    pub fn at_position(position: AbbreviationPosition) -> Self {
        Self {
            position,
            ..Self::default()
        }
    }

    pub fn with_command_patterns(
        position: AbbreviationPosition,
        command_patterns: Vec<String>,
    ) -> Result<Self, globset::Error> {
        let command_matchers = command_patterns
            .iter()
            .map(|pattern| Glob::new(pattern).map(|glob| glob.compile_matcher()))
            .collect::<Result<_, _>>()?;
        Ok(Self {
            position,
            command_patterns,
            command_matchers,
        })
    }

    fn matches_command(&self, command_context: Option<&str>) -> bool {
        self.command_matchers.is_empty()
            || command_context.is_some_and(|context| {
                context.len() <= MAX_COMMAND_CONTEXT_BYTES
                    && self
                        .command_matchers
                        .iter()
                        .any(|matcher| matcher.is_match(context))
            })
    }
}

#[derive(Debug, Deserialize)]
struct AbbreviationRaw {
    trigger: String,
    expansion: String,
    #[serde(default)]
    description: String,
    #[serde(default)]
    when: AbbreviationWhen,
}

#[derive(Debug, Clone, Deserialize, Default)]
pub struct KeybindingsRaw {
    pub tab: Option<String>,
    #[serde(rename = "shift-tab")]
    pub shift_tab: Option<String>,
    pub enter: Option<String>,
    pub space: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Default)]
struct ThemeRaw {
    border: Option<String>,
    #[serde(rename = "selected-fg")]
    selected_fg: Option<String>,
    #[serde(rename = "selected-bg")]
    selected_bg: Option<String>,
    #[serde(rename = "selected-marker")]
    selected_marker: Option<String>,
    description: Option<String>,
    filter: Option<String>,
    candidate: Option<String>,
    #[serde(rename = "match")]
    match_fg: Option<String>,
}

#[derive(Debug, Deserialize, Default)]
struct SuffixRaw {
    #[serde(flatten)]
    by_kind: HashMap<String, String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SuffixConfig {
    by_kind: HashMap<String, String>,
}

impl Default for SuffixConfig {
    fn default() -> Self {
        let mut by_kind = HashMap::new();
        by_kind.insert("directory".to_string(), "/".to_string());
        by_kind.insert("command".to_string(), " ".to_string());
        by_kind.insert("alias".to_string(), " ".to_string());
        by_kind.insert("builtin".to_string(), " ".to_string());
        by_kind.insert("function".to_string(), " ".to_string());
        by_kind.insert("file".to_string(), " ".to_string());
        Self { by_kind }
    }
}

impl SuffixConfig {
    fn from_raw(raw: SuffixRaw) -> Self {
        let mut suffixes = Self::default();
        for (kind, suffix) in raw.by_kind {
            suffixes.by_kind.insert(kind, suffix);
        }
        suffixes
    }

    pub fn suffix_for_kind(&self, kind: &str) -> Option<&str> {
        self.by_kind.get(kind).map(String::as_str)
    }

    pub fn with_override(mut self, kind: impl Into<String>, suffix: impl Into<String>) -> Self {
        self.by_kind.insert(kind.into(), suffix.into());
        self
    }
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
        "default" => Some(Color::Reset),
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

#[derive(Clone)]
pub struct Theme {
    pub border: Option<Color>,
    pub selected_fg: Option<Color>,
    pub selected_bg: Option<Color>,
    pub selected_marker: Color,
    pub description: Color,
    pub filter: Option<Color>,
    pub candidate: Option<Color>,
    pub match_fg: Color,
}

impl Default for Theme {
    fn default() -> Self {
        Theme {
            border: Some(Color::DarkGrey),
            selected_fg: None,
            selected_bg: None,
            selected_marker: Color::Cyan,
            description: Color::DarkGrey,
            filter: Some(Color::Cyan),
            candidate: None,
            match_fg: Color::Cyan,
        }
    }
}

impl Config {
    fn abbreviations_from_file(file: &ConfigFile) -> Vec<Abbreviation> {
        let mut by_trigger: BTreeMap<_, _> = file
            .abbreviations
            .iter()
            .map(|(trigger, expansion)| Abbreviation {
                trigger: trigger.clone(),
                expansion: expansion.clone(),
                description: String::new(),
                when: AbbreviationWhen::default(),
            })
            .map(|abbr| (abbr.trigger.clone(), abbr))
            .collect();
        for abbr in &file.abbreviation {
            by_trigger.insert(
                abbr.trigger.clone(),
                Abbreviation {
                    trigger: abbr.trigger.clone(),
                    expansion: abbr.expansion.clone(),
                    description: abbr.description.clone(),
                    when: abbr.when.clone(),
                },
            );
        }
        by_trigger.into_values().collect()
    }

    pub fn load() -> Self {
        let Some(path) = config_path() else {
            return Config::default();
        };
        Self::load_from(&path).unwrap_or_else(|error| {
            if error.kind() != io::ErrorKind::NotFound {
                tracing::warn!(path = %path.display(), %error, "failed to load config; using defaults");
            }
            Self::default()
        })
    }

    fn load_from(path: &Path) -> io::Result<Self> {
        let content = fs::read_to_string(path)?;
        let file: ConfigFile = toml::from_str(&content)
            .map_err(|error| io::Error::new(io::ErrorKind::InvalidData, error))?;
        let abbreviations = Self::abbreviations_from_file(&file);
        Ok(Config {
            max_visible: 10,
            auto_insert_unambiguous: file.completion.auto_insert_unambiguous,
            expand_abbreviation_on_enter: file.completion.expand_abbreviation_on_enter,
            expand_abbreviation_on_space: file.completion.expand_abbreviation_on_space,
            kind_order: file.completion.kind_order,
            suffixes: SuffixConfig::from_raw(file.suffix),
            keybindings: file.keybindings,
            abbreviations,
            theme_raw: file.theme,
        })
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
            border: Some(
                self.theme_raw
                    .border
                    .as_deref()
                    .and_then(parse_color)
                    .unwrap_or(Color::DarkGrey),
            ),
            selected_fg: self.theme_raw.selected_fg.as_deref().and_then(parse_color),
            selected_bg: self.theme_raw.selected_bg.as_deref().and_then(parse_color),
            selected_marker: self
                .theme_raw
                .selected_marker
                .as_deref()
                .and_then(parse_color)
                .unwrap_or(Color::Cyan),
            description: self
                .theme_raw
                .description
                .as_deref()
                .and_then(parse_color)
                .unwrap_or(Color::DarkGrey),
            filter: Some(
                self.theme_raw
                    .filter
                    .as_deref()
                    .and_then(parse_color)
                    .unwrap_or(Color::Cyan),
            ),
            candidate: self.theme_raw.candidate.as_deref().and_then(parse_color),
            match_fg: self
                .theme_raw
                .match_fg
                .as_deref()
                .and_then(parse_color)
                .unwrap_or(Color::Cyan),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn kind_order_defaults_and_partial_lists_keep_omitted_groups_in_default_order() {
        use crate::candidate::Candidate;
        let kinds = [
            "directory",
            "file",
            "abbreviation",
            "command",
            "alias",
            "builtin",
            "function",
            "other",
        ];
        for text in ["", "[completion]", "[completion]\nkind_order = []"] {
            let parsed: ConfigFile = toml::from_str(text).unwrap();
            assert_eq!(parsed.completion.kind_order, KindOrder::default());
        }
        let parsed: ConfigFile =
            toml::from_str("[completion]\nkind_order = ['command', 'file']").unwrap();
        for (kind, expected) in [
            ("command", 0),
            ("file", 1),
            ("directory", 2),
            ("abbreviation", 3),
            ("alias", 4),
            ("builtin", 5),
            ("function", 6),
            ("other", 7),
        ] {
            let candidate = Candidate::parse_line(&format!("name\t\t{kind}"));
            assert_eq!(
                parsed
                    .completion
                    .kind_order
                    .priority(candidate.kind_priority()),
                expected
            );
        }
        // Guard the mapping between configured names and the candidate model.
        for (expected, kind) in kinds.into_iter().enumerate() {
            let candidate = Candidate::parse_line(&format!("name\t\t{kind}"));
            assert_eq!(candidate.kind_priority() as usize, expected);
        }
    }

    #[test]
    fn kind_order_rejects_duplicates_unknown_names_and_invalid_types() {
        for value in [
            "['file', 'file']",
            "['files']",
            "['command_rescue']",
            "['']",
            "'file'",
            "[1]",
            "['directory', 'file', 'abbreviation', 'command', 'alias', 'builtin', 'function', 'other', 'file']",
        ] {
            let text = format!("[completion]\nkind_order = {value}");
            assert!(toml::from_str::<ConfigFile>(&text).is_err(), "{text}");
        }
    }

    #[test]
    fn kind_order_reload_recovers_from_invalid_values() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("config.toml");
        let mut reloader = ConfigReloader::new(path.clone());
        write_config(&path, "[completion]\nkind_order = ['command']", 1);
        assert_eq!(
            reloader.reload().unwrap().unwrap().kind_order,
            KindOrder::new(&["command"]).unwrap()
        );
        write_config(
            &path,
            "[completion]\nkind_order = ['command', 'command']",
            2,
        );
        assert_eq!(
            reloader.reload().err().unwrap().kind(),
            io::ErrorKind::InvalidData
        );
        assert!(reloader.reload().unwrap().is_none());
        write_config(&path, "[completion]\nkind_order = []", 3);
        assert_eq!(
            reloader.reload().unwrap().unwrap().kind_order,
            KindOrder::default()
        );
    }
    use crate::input::Action;

    #[test]
    fn config_path_prefers_absolute_xdg_config_home() {
        assert_eq!(
            config_path_from(
                Some(PathBuf::from("/custom/config")),
                Some(PathBuf::from("/home/alice")),
            ),
            Some(PathBuf::from("/custom/config/zacrs/config.toml"))
        );
    }

    #[test]
    fn config_path_defaults_to_dot_config_under_home() {
        assert_eq!(
            config_path_from(None, Some(PathBuf::from("/home/alice"))),
            Some(PathBuf::from("/home/alice/.config/zacrs/config.toml"))
        );
    }

    #[test]
    fn config_path_ignores_relative_xdg_config_home() {
        assert_eq!(
            config_path_from(
                Some(PathBuf::from("relative/config")),
                Some(PathBuf::from("/home/alice")),
            ),
            Some(PathBuf::from("/home/alice/.config/zacrs/config.toml"))
        );
    }

    #[test]
    fn config_path_is_none_without_xdg_config_home_or_home() {
        assert_eq!(config_path_from(None, None), None);
    }

    fn write_config(path: &Path, content: &str, modified: u64) {
        fs::write(path, content).unwrap();
        fs::File::open(path)
            .unwrap()
            .set_times(
                fs::FileTimes::new().set_modified(
                    SystemTime::UNIX_EPOCH + std::time::Duration::from_secs(modified),
                ),
            )
            .unwrap();
    }

    #[test]
    fn reload_reads_only_changed_versions_and_recovers_from_invalid_toml() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("config.toml");
        let mut reloader = ConfigReloader::new(path.clone());
        write_config(&path, "[abbreviations]\ngs = 'git status'", 1);
        let config = reloader.reload().unwrap().unwrap();
        assert_eq!(config.abbreviations[0].expansion, "git status");
        assert!(reloader.reload().unwrap().is_none());

        write_config(&path, "[abbreviations]\ngs = 'unfinished", 2);
        assert_eq!(
            reloader.reload().err().unwrap().kind(),
            io::ErrorKind::InvalidData
        );
        assert!(reloader.reload().unwrap().is_none());

        write_config(&path, "[abbreviations]\ngs = 'git status --short'", 3);
        let config = reloader.reload().unwrap().unwrap();
        assert_eq!(config.abbreviations[0].expansion, "git status --short");
    }

    #[test]
    fn reload_detects_length_changes_with_the_same_modification_time() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("config.toml");
        let mut reloader = ConfigReloader::new(path.clone());
        write_config(&path, "[abbreviations]\ngs = 'git status'", 1);
        assert!(reloader.reload().unwrap().is_some());
        write_config(&path, "[abbreviations]\ngs = 'git status --short'", 1);
        let config = reloader.reload().unwrap().unwrap();
        assert_eq!(config.abbreviations[0].expansion, "git status --short");
    }

    #[test]
    fn reload_retries_a_recreated_file_even_with_the_same_version() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("config.toml");
        let mut reloader = ConfigReloader::new(path.clone());
        assert!(reloader.reload().unwrap().is_none());
        write_config(&path, "[abbreviations]\ngs = 'git status'", 1);
        assert!(reloader.reload().unwrap().is_some());
        fs::remove_file(&path).unwrap();
        assert!(reloader.reload().unwrap().is_none());
        write_config(&path, "[abbreviations]\ngs = 'git branch'", 1);
        assert_eq!(
            reloader.reload().unwrap().unwrap().abbreviations[0].expansion,
            "git branch"
        );
    }

    #[test]
    fn reload_recovers_after_atomic_replacement() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("config.toml");
        let replacement = dir.path().join("replacement.toml");
        let mut reloader = ConfigReloader::new(path.clone());
        write_config(&path, "[abbreviations]\ngs = 'unfinished", 1);
        assert!(reloader.reload().is_err());
        write_config(&replacement, "[abbreviations]\ngs = 'git status'", 2);
        fs::rename(replacement, &path).unwrap();
        assert_eq!(
            reloader.reload().unwrap().unwrap().abbreviations[0].expansion,
            "git status"
        );
    }

    #[test]
    fn reload_retries_io_errors_instead_of_caching_them() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("config.toml");
        let mut reloader = ConfigReloader::new(path.clone());
        fs::create_dir(&path).unwrap();
        assert!(reloader.reload().is_err());
        assert!(reloader.reload().is_err());
        fs::remove_dir(&path).unwrap();
        write_config(&path, "", 1);
        assert!(reloader.reload().unwrap().unwrap().abbreviations.is_empty());
    }

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
            keybindings: KeybindingsRaw {
                tab: Some("confirm".to_string()),
                shift_tab: None,
                enter: None,
                space: Some("cancel".to_string()),
            },
            ..Config::default()
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
        assert_eq!(theme.border, Some(Color::DarkGrey));
        assert_eq!(theme.selected_fg, None);
        assert_eq!(theme.selected_bg, None);
        assert_eq!(theme.selected_marker, Color::Cyan);
        assert_eq!(theme.description, Color::DarkGrey);
        assert_eq!(theme.filter, Some(Color::Cyan));
        assert_eq!(theme.match_fg, Color::Cyan);
        assert_eq!(theme.candidate, None);
    }

    #[test]
    fn theme_overrides() {
        let config = Config {
            theme_raw: ThemeRaw {
                border: Some("blue".to_string()),
                selected_fg: Some("black".to_string()),
                selected_bg: Some("cyan".to_string()),
                selected_marker: Some("green".to_string()),
                description: Some("dark-grey".to_string()),
                filter: Some("green".to_string()),
                candidate: Some("white".to_string()),
                match_fg: Some("yellow".to_string()),
            },
            ..Config::default()
        };
        let theme = config.theme();
        assert_eq!(theme.border, Some(Color::Blue));
        assert_eq!(theme.selected_fg, Some(Color::Black));
        assert_eq!(theme.selected_bg, Some(Color::Cyan));
        assert_eq!(theme.selected_marker, Color::Green);
        assert_eq!(theme.description, Color::DarkGrey);
        assert_eq!(theme.filter, Some(Color::Green));
        assert_eq!(theme.candidate, Some(Color::White));
        assert_eq!(theme.match_fg, Color::Yellow);
    }

    #[test]
    fn space_expansion_is_independent_and_opt_in() {
        let file: ConfigFile =
            toml::from_str("[completion]\nexpand_abbreviation_on_space = true").unwrap();
        assert!(file.completion.expand_abbreviation_on_space);
        assert!(!file.completion.expand_abbreviation_on_enter);
        assert!(!Config::default().expand_abbreviation_on_space);
        assert!(
            toml::from_str::<ConfigFile>("[completion]\nexpand_abbreviation_on_space = 1").is_err()
        );
    }

    // --- auto_insert_unambiguous TOML parsing ---

    #[test]
    fn enter_expansion_is_opt_in_in_toml() {
        for content in [
            "",
            "[completion]",
            "[completion]\nexpand_abbreviation_on_enter = false",
        ] {
            let file: ConfigFile = toml::from_str(content).unwrap();
            assert!(!file.completion.expand_abbreviation_on_enter);
        }
        let file: ConfigFile =
            toml::from_str("[completion]\nexpand_abbreviation_on_enter = true").unwrap();
        assert!(file.completion.expand_abbreviation_on_enter);
        assert!(
            toml::from_str::<ConfigFile>("[completion]\nexpand_abbreviation_on_enter = 1").is_err()
        );
    }

    #[test]
    fn auto_insert_unambiguous_defaults_true_without_completion_section() {
        let file: ConfigFile = toml::from_str("").unwrap();
        assert!(file.completion.auto_insert_unambiguous);
    }

    #[test]
    fn auto_insert_unambiguous_defaults_true_when_key_absent() {
        let file: ConfigFile = toml::from_str("[completion]").unwrap();
        assert!(file.completion.auto_insert_unambiguous);
    }

    #[test]
    fn auto_insert_unambiguous_false_parsed_from_toml() {
        let file: ConfigFile =
            toml::from_str("[completion]\nauto_insert_unambiguous = false").unwrap();
        assert!(!file.completion.auto_insert_unambiguous);
    }

    #[test]
    fn auto_insert_unambiguous_true_explicit_parsed_from_toml() {
        let file: ConfigFile =
            toml::from_str("[completion]\nauto_insert_unambiguous = true").unwrap();
        assert!(file.completion.auto_insert_unambiguous);
    }

    #[test]
    fn auto_insert_unambiguous_malformed_toml_falls_back_to_default() {
        // toml::from_str fails → unwrap_or_default() → auto_insert_unambiguous = true
        let file: ConfigFile =
            toml::from_str("[completion]\nauto_insert_unambiguous = 42").unwrap_or_default();
        assert!(
            file.completion.auto_insert_unambiguous,
            "fallback default must be true"
        );
    }

    #[test]
    fn abbreviations_parse_shorthand_and_detailed_forms() {
        let file: ConfigFile = toml::from_str(
            r#"
[abbreviations]
gs = "git status"

[[abbreviation]]
trigger = "gcm"
expansion = "git commit -m '{{cursor}}'"
description = "commit with a message"

[[abbreviation]]
trigger = "null"
expansion = ">/dev/null"
when.position = "any"
"#,
        )
        .unwrap();

        assert_eq!(
            Config::abbreviations_from_file(&file),
            vec![
                Abbreviation {
                    trigger: "gcm".into(),
                    expansion: "git commit -m '{{cursor}}'".into(),
                    description: "commit with a message".into(),
                    when: AbbreviationWhen::default(),
                },
                Abbreviation {
                    trigger: "gs".into(),
                    expansion: "git status".into(),
                    description: String::new(),
                    when: AbbreviationWhen::default(),
                },
                Abbreviation {
                    trigger: "null".into(),
                    expansion: ">/dev/null".into(),
                    description: String::new(),
                    when: AbbreviationWhen {
                        position: AbbreviationPosition::Any,
                        ..Default::default()
                    },
                },
            ]
        );
    }

    #[test]
    fn abbreviation_when_command_accepts_one_or_many_globs() {
        let file: ConfigFile = toml::from_str(
            r#"
[[abbreviation]]
trigger = "null"
expansion = ">/dev/null"
when.position = "argument"
when.command = ["cargo *", "git add *"]
"#,
        )
        .unwrap();
        let abbreviations = Config::abbreviations_from_file(&file);
        let abbreviation = &abbreviations[0];

        assert!(abbreviation.is_available_at(false, Some("cargo test null")));
        assert!(abbreviation.is_available_at(false, Some("git add src null")));
        assert!(!abbreviation.is_available_at(false, Some("git commit null")));
        assert!(!abbreviation.is_available_at(true, Some("cargo test null")));
        assert!(!abbreviation.is_available_at(false, None));
    }

    #[test]
    fn abbreviation_when_command_glob_is_anchored() {
        let file: ConfigFile = toml::from_str(
            r#"
[[abbreviation]]
trigger = "null"
expansion = ">/dev/null"
when.position = "argument"
when.command = "cargo *"
"#,
        )
        .unwrap();
        let abbreviation = &Config::abbreviations_from_file(&file)[0];

        assert!(abbreviation.is_available_at(false, Some("cargo null")));
        assert!(!abbreviation.is_available_at(false, Some("sudo cargo null")));
    }

    #[test]
    fn abbreviation_when_rejects_invalid_command_glob() {
        let parsed = toml::from_str::<ConfigFile>(
            r#"
[[abbreviation]]
trigger = "null"
expansion = ">/dev/null"
when.command = "["
"#,
        );

        assert!(parsed.is_err());
    }

    #[test]
    fn suffix_defaults_match_current_behavior() {
        let suffixes = SuffixConfig::default();
        assert_eq!(suffixes.suffix_for_kind("directory"), Some("/"));
        assert_eq!(suffixes.suffix_for_kind("command"), Some(" "));
        assert_eq!(suffixes.suffix_for_kind("alias"), Some(" "));
        assert_eq!(suffixes.suffix_for_kind("builtin"), Some(" "));
        assert_eq!(suffixes.suffix_for_kind("function"), Some(" "));
        assert_eq!(suffixes.suffix_for_kind("file"), Some(" "));
        assert_eq!(suffixes.suffix_for_kind("other"), None);
    }

    #[test]
    fn suffix_overrides_parse_from_toml() {
        let file: ConfigFile =
            toml::from_str("[suffix]\ndirectory = \"\"\ncommand = \"!\"\nparameter = \"=\"\n")
                .unwrap();
        let suffixes = SuffixConfig::from_raw(file.suffix);
        assert_eq!(suffixes.suffix_for_kind("directory"), Some(""));
        assert_eq!(suffixes.suffix_for_kind("command"), Some("!"));
        assert_eq!(suffixes.suffix_for_kind("parameter"), Some("="));
        assert_eq!(suffixes.suffix_for_kind("file"), Some(" "));
    }

    #[test]
    fn theme_invalid_falls_back() {
        let config = Config {
            theme_raw: ThemeRaw {
                border: Some("invalid".to_string()),
                description: Some("also-invalid".to_string()),
                ..ThemeRaw::default()
            },
            ..Config::default()
        };
        let theme = config.theme();
        assert_eq!(theme.border, Some(Color::DarkGrey));
        assert_eq!(theme.description, Color::DarkGrey);
    }

    #[test]
    fn theme_can_use_terminal_default_for_accents() {
        let file: ConfigFile =
            toml::from_str("[theme]\nfilter = 'default'\nmatch = 'default'\nselected-marker = 'default'\nborder = 'default'").unwrap();
        let config = Config {
            theme_raw: file.theme,
            ..Config::default()
        };
        assert_eq!(config.theme().filter, Some(Color::Reset));
        assert_eq!(config.theme().match_fg, Color::Reset);
        assert_eq!(config.theme().selected_marker, Color::Reset);
        assert_eq!(config.theme().border, Some(Color::Reset));
    }
}
