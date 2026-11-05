use crate::{candidate::Candidate, config::Config, protocol::TextCompleteResult};

/// Resolve an exact command-position trigger without evaluating shell code.
pub fn expand_command_abbreviation(
    config: &Config,
    trigger: &str,
    command_context: &str,
) -> Option<TextCompleteResult> {
    let abbreviation = config.abbreviations.iter().find(|abbreviation| {
        abbreviation.trigger == trigger && abbreviation.is_available_at(true, Some(command_context))
    })?;
    let candidate = Candidate::abbreviation(
        abbreviation.trigger.clone(),
        abbreviation.expansion.clone(),
        abbreviation.description.clone(),
    );
    let text = candidate.insert_text?;
    if text.is_empty() {
        return None;
    }
    Some(TextCompleteResult {
        code: 0,
        text,
        chain: false,
        // Cursor placeholders describe an edit the user still needs to finish.
        execute: candidate.cursor_offset.is_none(),
        selection_applied: false,
        restore_text: String::new(),
        cursor_offset: candidate.cursor_offset,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{Abbreviation, AbbreviationPosition, AbbreviationWhen};

    fn config(expansion: &str, when: AbbreviationWhen) -> Config {
        let mut config = Config::default();
        config.abbreviations.push(Abbreviation {
            trigger: "gs".into(),
            expansion: expansion.into(),
            description: String::new(),
            when,
        });
        config
    }

    #[test]
    fn only_exact_triggers_expand() {
        let config = config("git status", AbbreviationWhen::default());
        let result = expand_command_abbreviation(&config, "gs", "gs -sb").unwrap();
        assert_eq!(result.text, "git status");
        assert!(result.execute);
        for trigger in ["g", "gss", "./gs", "'gs'", "GS"] {
            assert!(expand_command_abbreviation(&config, trigger, trigger).is_none());
        }
    }

    #[test]
    fn command_conditions_and_positions_are_respected() {
        let config = config(
            "git status",
            AbbreviationWhen::with_command_patterns(
                AbbreviationPosition::Command,
                vec!["gs -s*".into()],
            )
            .unwrap(),
        );
        assert!(expand_command_abbreviation(&config, "gs", "gs -sb").is_some());
        assert!(expand_command_abbreviation(&config, "gs", "gs").is_none());
        let config = self::config(
            "git status",
            AbbreviationWhen::at_position(AbbreviationPosition::Argument),
        );
        assert!(expand_command_abbreviation(&config, "gs", "gs").is_none());
    }

    #[test]
    fn cursor_placeholder_expands_without_executing() {
        let config = config("echo 日本語{{cursor}}", AbbreviationWhen::default());
        let result = expand_command_abbreviation(&config, "gs", "gs").unwrap();
        assert_eq!(result.text, "echo 日本語");
        assert_eq!(result.cursor_offset, Some(8));
        assert!(!result.execute);
    }

    #[test]
    fn expansion_is_literal_and_not_recursive() {
        let config = config("printf '%s\\n' \"$(pwd)\"\n", AbbreviationWhen::default());
        let result = expand_command_abbreviation(&config, "gs", "gs").unwrap();
        assert_eq!(result.text, config.abbreviations[0].expansion);
        let config = self::config("gs", AbbreviationWhen::default());
        assert_eq!(
            expand_command_abbreviation(&config, "gs", "gs")
                .unwrap()
                .text,
            "gs"
        );
    }
}
