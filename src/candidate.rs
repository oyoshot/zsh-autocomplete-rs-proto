use crate::config::SuffixConfig;
use std::collections::HashMap;

#[derive(Clone)]
pub struct Candidate {
    pub text: String,
    pub description: String,
    pub kind: String,
    pub insert_text: Option<String>,
    pub cursor_offset: Option<usize>,
}

impl Candidate {
    pub fn kind_priority(&self) -> u8 {
        match self.base_kind() {
            "directory" => 0,
            "file" => 1,
            "command" => 2,
            "alias" => 3,
            "builtin" => 4,
            "function" => 5,
            _ => 6,
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
            .then(|| self.text.strip_suffix('/'))
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
            && !self.text.ends_with('/')
            && !self.text.contains('/')
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
        if text.ends_with([' ', '/']) {
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
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
            "foo/"
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
        assert_eq!(c.text_with_suffix(&SuffixConfig::default()), "src/");
    }

    #[test]
    fn text_with_suffix_directory_already_slashed() {
        let c = Candidate::parse_line("src/\t\tdirectory");
        assert_eq!(c.text_with_suffix(&SuffixConfig::default()), "src/");
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

        assert_eq!(directory.text_with_suffix(&suffixes), "src/");
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
            "src/"
        );
    }

    #[test]
    fn text_for_dismiss_with_space_uses_command_override_for_empty_kind() {
        let c = Candidate::parse_line("git\t\t");
        let suffixes = SuffixConfig::default().with_override("command", "!");
        assert_eq!(c.text_for_dismiss_with_space(&suffixes, true), "git! ");
    }
}
