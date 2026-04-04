use crate::config::SuffixConfig;

#[derive(Clone)]
pub struct Candidate {
    pub text: String,
    pub description: String,
    pub kind: String,
}

impl Candidate {
    pub fn kind_priority(&self) -> u8 {
        match self.kind.as_str() {
            "directory" => 0,
            "file" => 1,
            "command" => 2,
            "alias" => 3,
            "builtin" => 4,
            "function" => 5,
            _ => 6,
        }
    }

    pub fn text_with_suffix(&self, suffixes: &SuffixConfig) -> String {
        let Some(suffix) = suffixes.suffix_for_kind(&self.kind) else {
            return self.text.clone();
        };
        if suffix.is_empty() || self.text.ends_with(suffix) {
            self.text.clone()
        } else {
            format!("{}{}", self.text, suffix)
        }
    }

    pub fn text_with_suffix_for_command_position(
        &self,
        suffixes: &SuffixConfig,
        is_command_position: bool,
    ) -> String {
        if is_command_position
            && self.kind.is_empty()
            && !self.text.ends_with('/')
            && !self.text.contains('/')
        {
            return format!("{} ", self.text);
        }

        self.text_with_suffix(suffixes)
    }

    pub fn text_for_dismiss_with_space(&self, suffixes: &SuffixConfig) -> String {
        let text = self.text_with_suffix(suffixes);
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
    fn text_with_suffix_command() {
        let c = Candidate::parse_line("git\t\tcommand");
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
            c.text_for_dismiss_with_space(&SuffixConfig::default()),
            "git "
        );
    }

    #[test]
    fn text_for_dismiss_with_space_directory_keeps_slash() {
        let c = Candidate::parse_line("src\t\tdirectory");
        assert_eq!(
            c.text_for_dismiss_with_space(&SuffixConfig::default()),
            "src/"
        );
    }
}
