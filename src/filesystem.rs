//! Portable, literal filesystem candidate collection. Shell syntax is not parsed here.
use crate::candidate::Candidate;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FilesystemQuery {
    pub cwd: PathBuf,
    /// Literal path used for lookup; any shell expansion has already been verified.
    pub path_prefix: String,
    /// Literal spelling used for candidates (may differ after home expansion).
    pub insertion_prefix: String,
}

impl FilesystemQuery {
    pub fn collect(&self) -> Vec<Candidate> {
        if !self.cwd.is_absolute()
            || self.path_prefix.contains('\0')
            || self.insertion_prefix.contains('\0')
        {
            return Vec::new();
        }
        let (directory, basename) = split_prefix(&self.path_prefix);
        let (insertion_directory, _) = split_prefix(&self.insertion_prefix);
        let lookup = if Path::new(directory).is_absolute() {
            PathBuf::from(directory)
        } else {
            self.cwd.join(directory)
        };
        let Ok(entries) = std::fs::read_dir(lookup) else {
            return Vec::new();
        };
        let mut candidates = Vec::new();
        for entry in entries.flatten() {
            // The engine uses UTF-8. Never replace invalid bytes with a different filename.
            let Ok(name) = entry.file_name().into_string() else {
                continue;
            };
            if !name.starts_with(basename) || (name.starts_with('.') && !basename.starts_with('.'))
            {
                continue;
            }
            let is_directory = entry.path().is_dir();
            let mut path = format!("{directory}{name}");
            let mut display_path = format!("{insertion_directory}{name}");
            if is_directory {
                path.push('/');
                display_path.push('/');
            }
            let mut text = String::with_capacity(display_path.len());
            for c in display_path.chars() {
                if c == '\\' {
                    text.push_str("\\\\");
                } else if c.is_control() {
                    text.extend(c.escape_default());
                } else {
                    text.push(c);
                }
            }
            candidates.push(Candidate {
                text,
                description: if is_directory { "directory" } else { "file" }.into(),
                kind: if is_directory { "directory" } else { "file" }.into(),
                insert_text: None,
                cursor_offset: None,
                literal_path: Some(path),
            });
        }
        candidates.sort_by(|a, b| a.text.cmp(&b.text));
        candidates
    }
}

fn split_prefix(prefix: &str) -> (&str, &str) {
    prefix
        .rfind('/')
        .map_or(("", prefix), |i| (&prefix[..=i], &prefix[i + 1..]))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn display_distinguishes_controls_from_literal_backslashes() {
        let temp = tempfile::tempdir().unwrap();
        for name in ["tab\tname", "tab\\tname", "line\nname", "line\\nname"] {
            std::fs::write(temp.path().join(name), "").unwrap();
        }
        let query = FilesystemQuery {
            cwd: temp.path().into(),
            path_prefix: String::new(),
            insertion_prefix: String::new(),
        };
        let rows = query.collect();
        let displays: std::collections::HashSet<_> = rows.iter().map(|c| c.text.as_str()).collect();
        assert_eq!(displays.len(), 4);
        assert!(
            rows.iter()
                .any(|c| c.text == "tab\\tname" && c.literal_path.as_deref() == Some("tab\tname"))
        );
        assert!(
            rows.iter().any(
                |c| c.text == "tab\\\\tname" && c.literal_path.as_deref() == Some("tab\\tname")
            )
        );
    }

    #[test]
    fn non_utf8_names_are_skipped_without_lossy_substitution() {
        use std::os::unix::ffi::OsStringExt;
        let temp = tempfile::tempdir().unwrap();
        std::fs::write(
            temp.path()
                .join(std::ffi::OsString::from_vec(vec![b'a', 0xff])),
            "",
        )
        .unwrap();
        let query = FilesystemQuery {
            cwd: temp.path().into(),
            path_prefix: String::new(),
            insertion_prefix: String::new(),
        };
        assert!(query.collect().is_empty());
    }

    #[test]
    fn literal_candidates_preserve_controls_and_classify_symlink_directories() {
        let temp = tempfile::tempdir().unwrap();
        for name in ["日本語 space", "tab\tname", "line\nname", ".hidden"] {
            std::fs::write(temp.path().join(name), "").unwrap();
        }
        std::fs::create_dir(temp.path().join("directory")).unwrap();
        std::os::unix::fs::symlink("directory", temp.path().join("link")).unwrap();
        let query = FilesystemQuery {
            cwd: temp.path().into(),
            path_prefix: String::new(),
            insertion_prefix: String::new(),
        };
        let rows = query.collect();
        assert_eq!(rows.len(), 5);
        assert!(rows.iter().any(|c| c.literal_path.as_deref()==Some("line\nname") && c.text=="line\\nname"));
        assert!(
            rows.iter()
                .any(|c| c.literal_path.as_deref() == Some("link/") && c.kind == "directory")
        );
        assert!(!rows.iter().any(|c| c.text.starts_with('.')));
    }
    #[test]
    fn cwd_and_prefix_are_request_local_and_literal() {
        let first = tempfile::tempdir().unwrap();
        let second = tempfile::tempdir().unwrap();
        std::fs::write(first.path().join("a*b"), "").unwrap();
        std::fs::write(second.path().join("another"), "").unwrap();
        let mut query = FilesystemQuery {
            cwd: first.path().into(),
            path_prefix: "a*".into(),
            insertion_prefix: "a*".into(),
        };
        assert_eq!(query.collect()[0].text, "a*b");
        query.cwd = second.path().into();
        assert!(query.collect().is_empty());
    }
}
