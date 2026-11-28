use crate::config::{Abbreviation, SuffixConfig};
use std::collections::HashMap;

#[derive(Clone)]
pub struct Candidate {
    pub text: String,
    pub description: String,
    pub kind: String,
    pub insert_text: Option<String>,
    pub cursor_offset: Option<usize>,
    pub source_group: SourceGroup,
}

/// Where a candidate's source placed it, for ordering empty queries.
///
/// Groups are listed by `index`. Within a group, candidates keep the order in
/// which the source listed them when `keep_order` is set. Otherwise they are
/// sorted by text, comparing runs of digits by value when `numeric` is set,
/// in descending order when `reverse` is set.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord)]
pub struct SourceGroup {
    pub index: u32,
    pub keep_order: bool,
    pub numeric: bool,
    pub reverse: bool,
}

impl SourceGroup {
    /// Parses a group number and a comma-separated order: `keep`, or any of
    /// `numeric` and `reverse`. Unknown orders are ignored.
    fn parse(index: Option<&str>, order: Option<&str>) -> Self {
        let mut group = Self {
            index: index.and_then(|i| i.parse().ok()).unwrap_or(0),
            ..Self::default()
        };
        for order in order.unwrap_or("").split(',') {
            match order {
                "keep" => group.keep_order = true,
                "numeric" => group.numeric = true,
                "reverse" => group.reverse = true,
                _ => {}
            }
        }
        group
    }
}

impl Candidate {
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

    /// Parses `text\tdescription\tkind\tgroup\torder`. Every field after
    /// the text is optional. `group` and `order` describe the source group
    /// (see [`SourceGroup`]); without them the candidate is in group 0,
    /// sorted by text.
    pub fn parse_line(line: &str) -> Self {
        let mut parts = line.splitn(5, '\t');
        let text = parts.next().unwrap_or("").to_string();
        let description = parts.next().unwrap_or("").to_string();
        let kind = parts.next().unwrap_or("").to_string();
        let source_group = SourceGroup::parse(parts.next(), parts.next());
        Candidate {
            text,
            description,
            kind,
            insert_text: None,
            cursor_offset: None,
            source_group,
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
            // Abbreviations keep the order of the configuration file.
            source_group: SourceGroup {
                keep_order: true,
                ..SourceGroup::default()
            },
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
        assert_eq!(c.source_group, SourceGroup::default());
    }

    #[test]
    fn parse_source_group() {
        let group = |line| Candidate::parse_line(line).source_group;
        let sorted = Candidate::parse_line("add\t\t\t2\t");
        assert_eq!(sorted.kind, "");
        assert_eq!(
            sorted.source_group,
            SourceGroup {
                index: 2,
                ..SourceGroup::default()
            }
        );
        assert_eq!(
            group("HEAD~1\t\t\t3\tkeep"),
            SourceGroup {
                index: 3,
                keep_order: true,
                ..SourceGroup::default()
            }
        );
        assert_eq!(
            group("10\t\t\t1\tnumeric,reverse"),
            SourceGroup {
                index: 1,
                numeric: true,
                reverse: true,
                ..SourceGroup::default()
            }
        );
    }

    #[test]
    fn parse_invalid_source_group_falls_back_to_the_default_group() {
        let c = Candidate::parse_line("add\t\t\tx\tunknown");
        assert_eq!(c.source_group, SourceGroup::default());
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
