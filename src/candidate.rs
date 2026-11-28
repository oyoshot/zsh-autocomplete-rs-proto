use crate::config::{Abbreviation, SuffixConfig};
use std::collections::HashMap;
use std::path::{MAIN_SEPARATOR_STR, Path, PathBuf, is_separator};

#[derive(Clone)]
pub struct Candidate {
    pub text: String,
    pub description: String,
    pub kind: String,
    pub insert_text: Option<String>,
    pub cursor_offset: Option<usize>,
    /// Filesystem identity without presentation separators or configured suffixes.
    pub literal_path: Option<LiteralPath>,
}

#[derive(Clone)]
pub struct LiteralPath {
    pub path: PathBuf,
    pub ignore_case: bool,
}

impl std::ops::Deref for LiteralPath {
    type Target = Path;

    fn deref(&self) -> &Path {
        &self.path
    }
}

#[cfg(test)]
impl From<PathBuf> for LiteralPath {
    fn from(path: PathBuf) -> Self {
        Self {
            path,
            ignore_case: false,
        }
    }
}

impl Candidate {
    pub(crate) fn filesystem_ignores_case(&self) -> bool {
        self.literal_path
            .as_ref()
            .is_some_and(|file| file.ignore_case)
    }

    /// Borrow the filesystem path and compute its separate shell insertion suffix.
    /// Default directory presentation follows the host; explicit suffixes remain policy.
    pub fn literal_insertion(
        &self,
        configured_suffix: Option<&str>,
        dismiss: bool,
    ) -> Option<(&Path, String)> {
        let path = self.literal_path.as_deref()?;
        let directory = self.base_kind() == "directory";
        let root = directory && path.has_root() && path.parent().is_none();
        let mut suffix = configured_suffix
            .unwrap_or(if directory { MAIN_SEPARATOR_STR } else { "" })
            .to_string();
        if root && suffix == MAIN_SEPARATOR_STR {
            suffix.clear();
        }
        // A quoted filename's trailing space is data, never a separator.
        if dismiss
            && !suffix.ends_with(|c| c == ' ' || is_separator(c))
            && !(root && suffix.is_empty())
        {
            suffix.push(' ');
        }
        Some((path, suffix))
    }

    pub fn kind_priority(&self) -> u8 {
        match self.base_kind() {
            "directory" => 0,
            "file" => 1,
            "abbreviation" => 2,
            "command" => 3,
            "alias" => 4,
            "builtin" => 5,
            "function" => 6,
            _ => 7,
        }
    }

    pub fn base_kind(&self) -> &str {
        self.kind.strip_suffix("_rescue").unwrap_or(&self.kind)
    }

    pub fn is_typo_rescue(&self) -> bool {
        self.kind.ends_with("_rescue")
    }

    pub fn is_user_defined(&self) -> bool {
        self.base_kind() == "abbreviation"
    }

    pub fn text_with_suffix(&self, suffixes: &SuffixConfig) -> String {
        let Some(suffix) = suffixes.suffix_for_kind(self.base_kind()) else {
            return self.text.clone();
        };
        let base_text = self
            .kind
            .eq("directory")
            .then(|| self.text.strip_suffix(is_separator))
            .flatten()
            .filter(|base| !base.is_empty())
            .unwrap_or(&self.text);

        if suffix.is_empty() || base_text.ends_with(suffix) {
            base_text.to_string()
        } else {
            format!("{}{}", base_text, suffix)
        }
    }

    pub fn text_with_suffix_for_command_position(
        &self,
        suffixes: &SuffixConfig,
        is_command_position: bool,
    ) -> String {
        if let Some(insert_text) = &self.insert_text {
            return insert_text.clone();
        }
        if is_command_position
            && self.kind.is_empty()
            && !self.text.ends_with(is_separator)
            && !self.text.contains(is_separator)
        {
            let command_suffix = suffixes.suffix_for_kind("command").unwrap_or(" ");
            if command_suffix.is_empty() || self.text.ends_with(command_suffix) {
                return self.text.clone();
            }
            return format!("{}{}", self.text, command_suffix);
        }

        self.text_with_suffix(suffixes)
    }

    pub fn text_for_dismiss_with_space(
        &self,
        suffixes: &SuffixConfig,
        is_command_position: bool,
    ) -> String {
        let text = self.text_with_suffix_for_command_position(suffixes, is_command_position);
        if text.ends_with(|c| c == ' ' || is_separator(c)) {
            text
        } else {
            format!("{text} ")
        }
    }

    pub fn parse_line(line: &str) -> Self {
        let mut parts = line.splitn(3, '\t');
        let text = parts.next().unwrap_or("").to_string();
        let description = parts.next().unwrap_or("").to_string();
        let kind = parts.next().unwrap_or("").to_string();
        Candidate {
            text,
            description,
            kind,
            insert_text: None,
            cursor_offset: None,
            literal_path: None,
        }
    }

    pub fn parse_lines_dedup(input: &str) -> Vec<Self> {
        let mut candidates: Vec<Self> = Vec::new();
        let mut indices_by_identity: HashMap<(String, String), usize> = HashMap::new();

        for line in input.lines().filter(|line| !line.is_empty()) {
            let candidate = Self::parse_line(line);
            let identity = (candidate.text.clone(), candidate.kind.clone());
            if let Some(&index) = indices_by_identity.get(&identity) {
                let existing = &mut candidates[index];
                if existing.description.is_empty() {
                    existing.description = candidate.description;
                }
            } else {
                indices_by_identity.insert(identity, candidates.len());
                candidates.push(candidate);
            }
        }

        candidates
    }

    pub fn abbreviation(trigger: String, expansion: String, description: String) -> Self {
        const CURSOR_MARKER: &str = "{{cursor}}";
        let (insert_text, cursor_offset) = match expansion.find(CURSOR_MARKER) {
            Some(offset) => (
                expansion.replacen(CURSOR_MARKER, "", 1),
                Some(expansion[..offset].chars().count()),
            ),
            None => (expansion, None),
        };
        let expansion_preview = insert_text
            .replace('\r', "\\r")
            .replace('\n', "\\n")
            .replace('\t', "\\t");
        let description = if description.is_empty() {
            expansion_preview
        } else {
            format!("{expansion_preview} — {description}")
        };
        Self {
            text: trigger,
            description,
            kind: "abbreviation".to_string(),
            insert_text: Some(insert_text),
            cursor_offset,
            literal_path: None,
        }
    }
}

impl From<&Abbreviation> for Candidate {
    fn from(abbreviation: &Abbreviation) -> Self {
        Self::abbreviation(
            abbreviation.trigger.clone(),
            abbreviation.expansion.clone(),
            abbreviation.description.clone(),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn literal_filename_trailing_space_never_consumes_a_separator() {
        let mut candidate = Candidate::parse_line("filename \tfile\tfile");
        candidate.literal_path = Some(PathBuf::from("filename ").into());
        assert_eq!(
            candidate.literal_insertion(Some(" "), false),
            Some((Path::new("filename "), " ".into()))
        );
        assert_eq!(
            candidate.literal_insertion(Some(" "), true),
            Some((Path::new("filename "), " ".into()))
        );
        assert_eq!(
            candidate.literal_insertion(Some(""), false),
            Some((Path::new("filename "), "".into()))
        );
        assert_eq!(
            candidate.literal_insertion(Some(""), true),
            Some((Path::new("filename "), " ".into()))
        );
    }

    #[test]
    fn literal_directory_suffixes_are_separate_from_the_path() {
        let mut candidate = Candidate::parse_line("./directory/\tdirectory\tdirectory");
        candidate.literal_path = Some(PathBuf::from("./directory").into());
        for (configured, dismiss, expected) in [
            (None, false, MAIN_SEPARATOR_STR),
            (None, true, MAIN_SEPARATOR_STR),
            (Some("/"), false, "/"),
            (Some("/"), true, "/"),
            (Some(""), false, ""),
            (Some(""), true, " "),
            (Some("!"), false, "!"),
            (Some("!"), true, "! "),
            (Some(" "), false, " "),
            (Some(" "), true, " "),
        ] {
            let (path, suffix) = candidate.literal_insertion(configured, dismiss).unwrap();
            assert_eq!(path.as_os_str(), std::ffi::OsStr::new("./directory"));
            assert_eq!(suffix, expected);
        }
        candidate.literal_path = Some(PathBuf::from(std::path::MAIN_SEPARATOR_STR).into());
        assert_eq!(candidate.literal_insertion(None, true).unwrap().1, "");
        assert_eq!(
            candidate
                .literal_insertion(Some(MAIN_SEPARATOR_STR), false)
                .unwrap()
                .1,
            ""
        );
        assert_eq!(
            candidate.literal_insertion(Some("!"), true).unwrap().1,
            "! "
        );
    }

    #[test]
    fn host_directory_separator_is_metadata_and_explicit_override_is_preserved() {
        let mut candidate = Candidate::parse_line("directory\t\tdirectory");
        candidate.literal_path = Some(PathBuf::from("directory").into());
        assert_eq!(
            candidate.literal_insertion(None, false).unwrap().1,
            MAIN_SEPARATOR_STR
        );
        assert_eq!(
            candidate
                .literal_insertion(Some(MAIN_SEPARATOR_STR), true)
                .unwrap()
                .1,
            MAIN_SEPARATOR_STR
        );
        assert_eq!(candidate.literal_insertion(Some("/"), true).unwrap().1, "/");
        assert_eq!(
            candidate.literal_insertion(Some("\\"), true).unwrap().1,
            if is_separator('\\') { "\\" } else { "\\ " }
        );
        let suffixed =
            Candidate::parse_line(&format!("directory{MAIN_SEPARATOR_STR}\t\tdirectory"));
        assert_eq!(
            suffixed.text_with_suffix(&SuffixConfig::default().with_override("directory", "")),
            "directory"
        );
    }

    #[test]
    fn parse_with_description_and_kind() {
        let c = Candidate::parse_line("git\tcommand\tcommand");
        assert_eq!(c.text, "git");
        assert_eq!(c.description, "command");
        assert_eq!(c.kind, "command");
    }

    #[test]
    fn parse_with_description_only() {
        let c = Candidate::parse_line("git\tcommand");
        assert_eq!(c.text, "git");
        assert_eq!(c.description, "command");
        assert_eq!(c.kind, "");
    }

    #[test]
    fn parse_without_description() {
        let c = Candidate::parse_line("git");
        assert_eq!(c.text, "git");
        assert_eq!(c.description, "");
        assert_eq!(c.kind, "");
    }

    #[test]
    fn parse_three_fields() {
        let c = Candidate::parse_line("src/\tdirectory\tdirectory");
        assert_eq!(c.text, "src/");
        assert_eq!(c.description, "directory");
        assert_eq!(c.kind, "directory");
    }

    #[test]
    fn parse_lines_dedup_keeps_first_occurrence_order() {
        let candidates = Candidate::parse_lines_dedup(
            "src/main.rs\tmodified\tfile\nCargo.toml\tmodified\tfile\nsrc/main.rs\tuntracked\tfile\n",
        );

        assert_eq!(candidates.len(), 2);
        assert_eq!(candidates[0].text, "src/main.rs");
        assert_eq!(candidates[0].description, "modified");
        assert_eq!(candidates[1].text, "Cargo.toml");
    }

    #[test]
    fn parse_lines_dedup_fills_missing_metadata_from_duplicate() {
        let candidates =
            Candidate::parse_lines_dedup("src/main.rs\t\tfile\nsrc/main.rs\tmodified\tfile\n");

        assert_eq!(candidates.len(), 1);
        assert_eq!(candidates[0].description, "modified");
        assert_eq!(candidates[0].kind, "file");
    }

    #[test]
    fn parse_lines_dedup_preserves_different_insertion_semantics() {
        let candidates =
            Candidate::parse_lines_dedup("foo\tdirectory\tdirectory\nfoo\tcommand\tcommand\n");

        assert_eq!(candidates.len(), 2);
        assert_eq!(
            candidates[0].text_with_suffix(&SuffixConfig::default()),
            format!("foo{MAIN_SEPARATOR_STR}")
        );
        assert_eq!(
            candidates[1].text_with_suffix(&SuffixConfig::default()),
            "foo "
        );
    }

    #[test]
    fn abbreviation_separates_trigger_from_insert_text() {
        let c = Candidate::abbreviation(
            "gcm".to_string(),
            "git commit -m '{{cursor}}'".to_string(),
            "commit with a message".to_string(),
        );
        assert_eq!(c.text, "gcm");
        assert_eq!(c.insert_text.as_deref(), Some("git commit -m ''"));
        assert_eq!(c.cursor_offset, Some(15));
        assert_eq!(c.description, "git commit -m '' — commit with a message");
        assert_eq!(c.kind, "abbreviation");
    }

    #[test]
    fn abbreviation_uses_expansion_as_description_when_label_is_absent() {
        let c = Candidate::abbreviation("gs".into(), "git status".into(), String::new());
        assert_eq!(c.description, "git status");
    }

    #[test]
    fn abbreviation_escapes_control_whitespace_in_description() {
        let c = Candidate::abbreviation("multi".into(), "printf 'a\tb\n'".into(), String::new());
        assert_eq!(c.description, "printf 'a\\tb\\n'");
    }

    #[test]
    fn abbreviation_dismiss_with_space_expands_and_appends_space() {
        let c = Candidate::abbreviation("gs".into(), "git status".into(), String::new());
        assert_eq!(
            c.text_for_dismiss_with_space(&SuffixConfig::default(), true),
            "git status "
        );
    }

    #[test]
    fn text_with_suffix_directory() {
        let c = Candidate::parse_line("src\t\tdirectory");
        assert_eq!(
            c.text_with_suffix(&SuffixConfig::default()),
            format!("src{MAIN_SEPARATOR_STR}")
        );
    }

    #[test]
    fn text_with_suffix_directory_already_slashed() {
        let c = Candidate::parse_line("src/\t\tdirectory");
        assert_eq!(
            c.text_with_suffix(&SuffixConfig::default()),
            format!("src{MAIN_SEPARATOR_STR}")
        );
    }

    #[test]
    fn text_with_suffix_directory_empty_override_removes_trailing_slash() {
        let c = Candidate::parse_line("src/\t\tdirectory");
        let suffixes = SuffixConfig::default().with_override("directory", "");
        assert_eq!(c.text_with_suffix(&suffixes), "src");
    }

    #[test]
    fn text_with_suffix_directory_custom_override_replaces_trailing_slash() {
        let c = Candidate::parse_line("src/\t\tdirectory");
        let suffixes = SuffixConfig::default().with_override("directory", " ");
        assert_eq!(c.text_with_suffix(&suffixes), "src ");
    }

    #[test]
    fn text_with_suffix_command() {
        let c = Candidate::parse_line("git\t\tcommand");
        assert_eq!(c.text_with_suffix(&SuffixConfig::default()), "git ");
    }

    #[test]
    fn text_with_suffix_file_keeps_path_and_appends_file_suffix() {
        let c = Candidate::parse_line("notes.txt\t\tfile");
        assert_eq!(c.text_with_suffix(&SuffixConfig::default()), "notes.txt ");
    }

    #[test]
    fn path_kinds_keep_file_and_directory_suffixes_distinct() {
        let directory = Candidate::parse_line("src\t\tdirectory");
        let file = Candidate::parse_line("src/main.rs\t\tfile");
        let suffixes = SuffixConfig::default();

        assert_eq!(
            directory.text_with_suffix(&suffixes),
            format!("src{MAIN_SEPARATOR_STR}")
        );
        assert_eq!(file.text_with_suffix(&suffixes), "src/main.rs ");
    }

    #[test]
    fn text_with_suffix_command_rescue_uses_command_suffix() {
        let c = Candidate::parse_line("git\t\tcommand_rescue");
        let command = Candidate::parse_line("git\t\tcommand");
        assert_eq!(c.kind_priority(), command.kind_priority());
        assert_eq!(c.text_with_suffix(&SuffixConfig::default()), "git ");
    }

    #[test]
    fn text_with_suffix_unknown_kind() {
        let c = Candidate::parse_line("foo\t\tother");
        assert_eq!(c.text_with_suffix(&SuffixConfig::default()), "foo");
    }

    #[test]
    fn text_with_suffix_uses_custom_config() {
        let c = Candidate::parse_line("git\t\tcommand");
        let custom = SuffixConfig::default().with_override("command", "!");
        assert_eq!(c.text_with_suffix(&custom), "git!");
    }

    #[test]
    fn text_with_suffix_for_command_position_adds_space_for_empty_kind() {
        let c = Candidate::parse_line("git\t\t");
        assert_eq!(
            c.text_with_suffix_for_command_position(&SuffixConfig::default(), true),
            "git "
        );
    }

    #[test]
    fn text_with_suffix_for_command_position_uses_command_override_for_empty_kind() {
        let c = Candidate::parse_line("git\t\t");
        let suffixes = SuffixConfig::default().with_override("command", "!");
        assert_eq!(
            c.text_with_suffix_for_command_position(&suffixes, true),
            "git!"
        );
    }

    #[test]
    fn text_with_suffix_for_command_position_honors_empty_command_override() {
        let c = Candidate::parse_line("git\t\t");
        let suffixes = SuffixConfig::default().with_override("command", "");
        assert_eq!(
            c.text_with_suffix_for_command_position(&suffixes, true),
            "git"
        );
    }

    #[test]
    fn text_with_suffix_for_command_position_keeps_paths_without_space() {
        let c = Candidate::parse_line("./script\t\t");
        assert_eq!(
            c.text_with_suffix_for_command_position(&SuffixConfig::default(), true),
            "./script"
        );
    }

    #[test]
    fn text_for_dismiss_with_space_unknown_kind() {
        let c = Candidate::parse_line("git\t\t");
        assert_eq!(
            c.text_for_dismiss_with_space(&SuffixConfig::default(), false),
            "git "
        );
    }

    #[test]
    fn text_for_dismiss_with_space_directory_keeps_slash() {
        let c = Candidate::parse_line("src\t\tdirectory");
        assert_eq!(
            c.text_for_dismiss_with_space(&SuffixConfig::default(), false),
            format!("src{MAIN_SEPARATOR_STR}")
        );
    }

    #[test]
    fn text_for_dismiss_with_space_uses_command_override_for_empty_kind() {
        let c = Candidate::parse_line("git\t\t");
        let suffixes = SuffixConfig::default().with_override("command", "!");
        assert_eq!(c.text_for_dismiss_with_space(&suffixes, true), "git! ");
    }
}
