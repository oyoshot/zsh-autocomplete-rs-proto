use crate::{candidate::Candidate, config::Config, protocol::TextCompleteResult};

fn bare_command(buffer: &str) -> Option<(&str, usize)> {
    let trimmed = buffer.trim_start();
    let end = trimmed
        .find(|c: char| !(c.is_alphanumeric() || "_.+-".contains(c)))
        .unwrap_or(trimmed.len());
    let word = &trimmed[..end];
    if word.is_empty() || word == "." || word == ".." {
        return None;
    }
    Some((word, buffer.len() - trimmed.len()))
}

// Events contain observations, not shell-side policy. Replay command changes so
// deleting a selected command and typing it again does not restore its exemption.
fn selected_command(events: &[String]) -> Option<String> {
    let mut selected = None;
    for event in events {
        if let Some(word) = event.strip_prefix("edit:") {
            if selected.as_deref() != Some(word) {
                selected = None;
            }
        } else if let Some(event) = event.strip_prefix("select:") {
            let mut fields = event.splitn(3, ':');
            let applied = fields.next() == Some("1");
            let start = fields.next()?.parse::<usize>().ok()?;
            let buffer = fields.next()?;
            let Some((word, leading)) = bare_command(buffer) else {
                selected = None;
                continue;
            };
            if selected.as_deref() != Some(word) {
                selected = None;
            }
            if start <= buffer[..leading].chars().count() {
                selected = applied.then(|| word.to_owned());
            }
        }
    }
    selected
}

/// Plan an Enter edit from ZLE's buffer and Zsh's raw lexical tokens.
/// No shell code is evaluated here. Unsupported syntax passes through unchanged.
pub fn expand_enter_line(
    config: &Config,
    buffer: &str,
    prebuffer: &str,
    tokens: &[String],
    events: &[String],
    command_context: &str,
) -> Option<TextCompleteResult> {
    if !config.expand_abbreviation_on_enter || !prebuffer.is_empty() {
        return None;
    }
    let (word, leading) = bare_command(buffer)?;
    if tokens.first().map(String::as_str) != Some(word)
        || selected_command(events).as_deref() == Some(word)
        || matches!(
            word,
            "if" | "then"
                | "elif"
                | "else"
                | "fi"
                | "for"
                | "foreach"
                | "while"
                | "until"
                | "repeat"
                | "do"
                | "done"
                | "case"
                | "esac"
                | "select"
                | "function"
                | "coproc"
                | "time"
                | "nocorrect"
                | "noglob"
                | "command"
                | "exec"
                | "builtin"
                | "end"
        )
        || tokens.iter().any(|t| {
            matches!(
                t.as_str(),
                "|" | "|&"
                    | "||"
                    | "&&"
                    | "&"
                    | "&!"
                    | "&|"
                    | ";"
                    | "\n"
                    | "("
                    | ")"
                    | "()"
                    | "{"
                    | "}"
            )
        })
    {
        return None;
    }
    let mut result = expand_command_abbreviation(config, word, command_context)?;
    result.cursor_offset = result
        .cursor_offset
        .map(|offset| buffer[..leading].chars().count() + offset);
    result.text = format!(
        "{}{}{}",
        &buffer[..leading],
        result.text,
        &buffer[leading + word.len()..]
    );
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

    fn enter(
        buffer: &str,
        prebuffer: &str,
        tokens: &[&str],
        events: &[&str],
    ) -> Option<TextCompleteResult> {
        let mut config = config("git status", AbbreviationWhen::default());
        config.expand_abbreviation_on_enter = true;
        expand_enter_line(
            &config,
            buffer,
            prebuffer,
            &tokens.iter().map(|s| (*s).into()).collect::<Vec<_>>(),
            &events.iter().map(|s| (*s).into()).collect::<Vec<_>>(),
            buffer,
        )
    }

    #[test]
    fn enter_preserves_suffix_and_requires_simple_bare_command() {
        assert_eq!(
            enter(
                "  gs\t-sb \"日本語 a\"  ",
                "",
                &["gs", "-sb", "\"日本語 a\""],
                &[]
            )
            .unwrap()
            .text,
            "  git status\t-sb \"日本語 a\"  "
        );
        assert!(enter("gs", "cat <<'EOF'\n", &["gs"], &[]).is_none());
        for word in ["", "./gs", "'gs'", "\\gs", "A=1", "command", "function"] {
            assert!(enter(word, "", &[word], &[]).is_none(), "{word}");
        }
        for operator in [
            "|", "|&", "||", "&&", "&", "&!", "&|", ";", "\n", "(", ")", "()", "{", "}",
        ] {
            assert!(
                enter(
                    &format!("gs {operator} echo done"),
                    "",
                    &["gs", operator, "echo", "done"],
                    &[]
                )
                .is_none(),
                "{operator}"
            );
        }
        assert!(enter("gs \"()\"", "", &["gs", "\"()\""], &[]).is_some());
        assert_eq!(
            enter("gs>out", "", &["gs", ">", "out"], &[]).unwrap().text,
            "git status>out"
        );
        assert!(enter("gs() print hello", "", &["gs", "()", "print", "hello"], &[]).is_none());
    }

    #[test]
    fn explicit_choices_survive_arguments_but_not_command_edits_or_dismissal() {
        assert!(enter("gs -sb", "", &["gs", "-sb"], &["select:1:0:gs ", "edit:gs"]).is_none());
        assert!(
            enter(
                "gs new",
                "",
                &["gs", "new"],
                &["select:1:0:gs ", "select:1:3:gs new"]
            )
            .is_none()
        );
        for events in [
            vec!["select:0:0:gs "],
            vec!["select:1:0:gs ", "select:0:0:gs "],
            vec!["select:1:3:gs new"],
            vec!["select:1:0:gs ", "edit:g", "edit:gs"],
            vec!["select:1:0:gs ", "edit:gsx", "edit:gs"],
        ] {
            assert!(
                enter("gs new", "", &["gs", "new"], &events).is_some(),
                "{events:?}"
            );
        }
    }

    #[test]
    fn enter_default_is_disabled_and_cursor_offset_counts_characters() {
        let mut config = config("git {{cursor}}", AbbreviationWhen::default());
        let tokens = vec!["gs".into()];
        assert!(expand_enter_line(&config, "gs", "", &tokens, &[], "gs").is_none());
        config.expand_abbreviation_on_enter = true;
        let result = expand_enter_line(&config, "\u{3000}gs", "", &tokens, &[], "gs").unwrap();
        assert_eq!(result.text, "\u{3000}git ");
        assert_eq!(result.cursor_offset, Some(5));
        assert!(!result.execute);
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
