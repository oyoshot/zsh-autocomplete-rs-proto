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

    pub fn text_with_suffix(&self) -> String {
        let suffix = match self.kind.as_str() {
            "directory" => {
                if self.text.ends_with('/') {
                    ""
                } else {
                    "/"
                }
            }
            "command" | "alias" | "builtin" | "function" | "file" => " ",
            _ => "",
        };
        format!("{}{}", self.text, suffix)
    }

    pub fn text_for_dismiss_with_space(&self) -> String {
        let text = self.text_with_suffix();
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
        assert_eq!(c.text_with_suffix(), "src/");
    }

    #[test]
    fn text_with_suffix_directory_already_slashed() {
        let c = Candidate::parse_line("src/\t\tdirectory");
        assert_eq!(c.text_with_suffix(), "src/");
    }

    #[test]
    fn text_with_suffix_command() {
        let c = Candidate::parse_line("git\t\tcommand");
        assert_eq!(c.text_with_suffix(), "git ");
    }

    #[test]
    fn text_with_suffix_unknown_kind() {
        let c = Candidate::parse_line("foo\t\tother");
        assert_eq!(c.text_with_suffix(), "foo");
    }

    #[test]
    fn text_for_dismiss_with_space_unknown_kind() {
        let c = Candidate::parse_line("git\t\t");
        assert_eq!(c.text_for_dismiss_with_space(), "git ");
    }

    #[test]
    fn text_for_dismiss_with_space_directory_keeps_slash() {
        let c = Candidate::parse_line("src\t\tdirectory");
        assert_eq!(c.text_for_dismiss_with_space(), "src/");
    }
}
