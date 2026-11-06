use crate::{candidate::Candidate, config::Config, protocol::TextCompleteResult};

/// A command word and replacement range verified by a shell adapter.
/// Offsets count Unicode characters, matching the existing cursor edit protocol.
#[derive(Debug, Clone)]
pub struct CommandTarget {
    pub trigger: String,
    pub start: usize,
    pub end: usize,
    pub command_context: String,
}

/// Shell-independent selection protection. Word identity is supplied by the
/// adapter's native lexer; this module never interprets shell syntax.
#[derive(Debug, Default, Clone)]
pub struct EditingSession {
    selected: Option<String>,
    uncertain: bool,
}

impl EditingSession {
    pub fn observe(&mut self, head: Option<&str>) {
        match head {
            None => {
                self.selected = None;
                self.uncertain = true;
            }
            Some(head) if self.selected.as_deref() != Some(head) => self.selected = None,
            _ => {}
        }
    }

    pub fn select(
        &mut self,
        edit_start: usize,
        applied: bool,
        head: Option<&str>,
        head_start: usize,
    ) {
        self.observe(head);
        if edit_start <= head_start {
            self.selected = head
                .filter(|head| applied && !head.is_empty())
                .map(str::to_owned);
        }
    }
}

/// Apply a verified literal range. Shell grammar and syntax capabilities belong
/// to the adapter; invalid or unavailable ranges always preserve the input.
pub fn expand_enter_line(
    config: &Config,
    buffer: &str,
    target: Option<&CommandTarget>,
    session: &EditingSession,
) -> Option<TextCompleteResult> {
    if !config.expand_abbreviation_on_enter || session.uncertain {
        return None;
    }
    let target = target?;
    if target.trigger.is_empty()
        || session.selected.as_deref() == Some(&target.trigger)
        || target.start >= target.end
    {
        return None;
    }
    let byte_at = |offset: usize| {
        buffer
            .char_indices()
            .map(|(i, _)| i)
            .chain(std::iter::once(buffer.len()))
            .nth(offset)
    };
    let start = byte_at(target.start)?;
    let end = byte_at(target.end)?;
    if buffer.get(start..end)? != target.trigger {
        return None;
    }
    let mut result = expand_command_abbreviation(config, &target.trigger, &target.command_context)?;
    result.cursor_offset = result.cursor_offset.map(|offset| target.start + offset);
    result.text = format!("{}{}{}", &buffer[..start], result.text, &buffer[end..]);
    Some(result)
}

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

    #[test]
    fn verified_ranges_preserve_arguments_and_validate_offsets() {
        let mut config = config("git {{cursor}}", AbbreviationWhen::default());
        let buffer = "　gs -sb";
        let mut target = CommandTarget {
            trigger: "gs".into(),
            start: 1,
            end: 3,
            command_context: "gs -sb".into(),
        };
        let session = EditingSession::default();
        assert!(expand_enter_line(&config, buffer, Some(&target), &session).is_none());
        config.expand_abbreviation_on_enter = true;
        let result = expand_enter_line(&config, buffer, Some(&target), &session).unwrap();
        assert_eq!(result.text, "　git  -sb");
        assert_eq!(result.cursor_offset, Some(5));
        assert!(expand_enter_line(&config, buffer, None, &session).is_none());
        target.end = 20;
        assert!(expand_enter_line(&config, buffer, Some(&target), &session).is_none());
        target.end = 2;
        assert!(expand_enter_line(&config, buffer, Some(&target), &session).is_none());
    }

    #[test]
    fn native_word_observations_preserve_choices_and_unknown_context_is_safe() {
        let mut session = EditingSession::default();
        session.select(0, true, Some("gs"), 0);
        session.observe(Some("gs"));
        assert_eq!(session.selected.as_deref(), Some("gs"));
        session.select(3, true, Some("gs"), 0);
        assert_eq!(session.selected.as_deref(), Some("gs"));
        session.observe(Some("gsx"));
        session.observe(Some("gs"));
        assert!(session.selected.is_none());
        session.observe(None);
        session.select(0, true, Some("gs"), 0);
        assert!(session.uncertain);
    }

    #[test]
    fn shared_engine_does_not_classify_a_shells_reserved_words() {
        let mut config = config("literal edit", AbbreviationWhen::default());
        config.expand_abbreviation_on_enter = true;
        config.abbreviations[0].trigger = "if".into();
        let target = CommandTarget {
            trigger: "if".into(),
            start: 0,
            end: 2,
            command_context: "if".into(),
        };
        assert_eq!(
            expand_enter_line(&config, "if", Some(&target), &EditingSession::default())
                .unwrap()
                .text,
            "literal edit"
        );
        // Zsh refuses this target in its native adapter; another backend is
        // responsible for establishing its own capabilities before offering it.
    }

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
