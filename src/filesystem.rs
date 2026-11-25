//! Host-native filesystem candidate collection.
//! Shell syntax and display spelling remain adapter-supplied facts.
use crate::candidate::Candidate;
use std::path::{Component, MAIN_SEPARATOR, Path, PathBuf, is_separator};
use unicode_casefold::{Locale, UnicodeCaseFold, Variant};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FilesystemQuery {
    pub cwd: PathBuf,
    /// Literal path used for lookup; any shell expansion has already been verified.
    pub path_prefix: String,
    /// Literal spelling used for candidates (may differ after home expansion).
    pub insertion_prefix: String,
    pub ignore_case: bool,
}

const MAX_SNAPSHOT_BYTES: usize = 1_048_576;

#[derive(Debug, PartialEq, Eq)]
pub struct SnapshotTooLarge;

impl FilesystemQuery {
    pub fn collect(&self) -> Result<Vec<Candidate>, SnapshotTooLarge> {
        self.collect_with_budget(MAX_SNAPSHOT_BYTES)
    }

    fn collect_with_budget(&self, budget: usize) -> Result<Vec<Candidate>, SnapshotTooLarge> {
        if !self.cwd.is_absolute()
            || self.path_prefix.contains('\0')
            || self.insertion_prefix.contains('\0')
        {
            return Ok(Vec::new());
        }
        if !supports_lookup_prefix(Path::new(&self.path_prefix)) {
            return Ok(Vec::new());
        }
        let (directory, basename) = split_prefix(&self.path_prefix);
        let directory = Path::new(directory);
        let (insertion_directory, _) = split_prefix(&self.insertion_prefix);
        let lookup = self.cwd.join(directory);
        let Ok(entries) = std::fs::read_dir(lookup.as_path()) else {
            return Ok(Vec::new());
        };
        let folded_prefix: Vec<_> = if self.ignore_case {
            basename
                .case_fold_with(Variant::Simple, Locale::NonTurkic)
                .collect()
        } else {
            Vec::new()
        };
        let mut candidates = Vec::new();
        let mut retained_bytes = 0;
        for entry in entries.flatten() {
            // The engine uses UTF-8. Never replace invalid bytes with a different filename.
            let name = entry.file_name();
            let Some(name_text) = name.to_str() else {
                continue;
            };
            if name_text.starts_with('.') && !basename.starts_with('.') {
                continue;
            }
            if !name_text.starts_with(basename)
                && !(self.ignore_case && {
                    let mut folded_name =
                        name_text.case_fold_with(Variant::Simple, Locale::NonTurkic);
                    folded_prefix.iter().all(|&c| folded_name.next() == Some(c))
                })
            {
                continue;
            }
            let is_directory = entry
                .file_type()
                .is_ok_and(|kind| kind.is_dir() || (kind.is_symlink() && entry.path().is_dir()));
            let path = directory.join(&name);
            let mut display_path = format!("{insertion_directory}{name_text}");
            if is_directory {
                // Presentation keeps its separator separate from the actual path.
                display_path.push(MAIN_SEPARATOR);
            }
            let mut text = String::with_capacity(display_path.len());
            for c in display_path.chars() {
                if c == '\\' && !is_separator(c) {
                    text.push_str("\\\\");
                } else if c.is_control() {
                    text.extend(c.escape_default());
                } else {
                    text.push(c);
                }
            }
            let candidate = Candidate {
                text,
                description: if is_directory { "directory" } else { "file" }.into(),
                kind: if is_directory { "directory" } else { "file" }.into(),
                insert_text: None,
                cursor_offset: None,
                literal_path: Some(path),
            };
            // Count owned data as well as row storage. Abandon the whole query
            // on overflow, since a partial snapshot could be auto-selected.
            retained_bytes += size_of::<Candidate>()
                + candidate.text.capacity()
                + candidate.description.capacity()
                + candidate.kind.capacity()
                + candidate.literal_path.as_ref().unwrap().capacity();
            if retained_bytes > budget {
                return Err(SnapshotTooLarge);
            }
            candidates.push(candidate);
        }
        candidates.sort_by(|a, b| a.text.cmp(&b.text));
        Ok(candidates)
    }
}

// A drive-qualified relative path requires per-drive shell state that this
// verified cwd-relative/absolute query does not supply. Decide before I/O.
fn supports_lookup_prefix(prefix: &Path) -> bool {
    !matches!(prefix.components().next(), Some(Component::Prefix(_))) || prefix.is_absolute()
}

fn split_prefix(prefix: &str) -> (&str, &str) {
    prefix
        .rfind(is_separator)
        .map_or(("", prefix), |i| (&prefix[..=i], &prefix[i + 1..]))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn case_insensitive_prefixes_preserve_the_names_on_disk() {
        let root = tempfile::tempdir().unwrap();
        std::fs::write(root.path().join("Ωmega"), "").unwrap();
        std::fs::write(root.path().join(".Ωmega"), "").unwrap();
        let mut query = FilesystemQuery {
            cwd: root.path().into(),
            path_prefix: "ωM".into(),
            insertion_prefix: "ωM".into(),
            ignore_case: false,
        };
        assert!(query.collect().unwrap().is_empty());
        query.ignore_case = true;
        let candidates = query.collect().unwrap();
        assert_eq!(candidates.len(), 1);
        assert_eq!(candidates[0].text, "Ωmega");
        assert_eq!(
            candidates[0].literal_path.as_deref(),
            Some(Path::new("Ωmega"))
        );
    }

    #[test]
    fn oversized_snapshot_is_rejected_as_a_whole() {
        let root = tempfile::tempdir().unwrap();
        let query = FilesystemQuery {
            cwd: root.path().into(),
            path_prefix: String::new(),
            insertion_prefix: String::new(),
            ignore_case: false,
        };
        let budget = size_of::<Candidate>() + 64;
        std::fs::write(root.path().join("first"), "").unwrap();
        assert_eq!(query.collect_with_budget(budget).unwrap().len(), 1);
        std::fs::write(root.path().join("other"), "").unwrap();
        assert!(matches!(
            query.collect_with_budget(budget),
            Err(SnapshotTooLarge)
        ));
    }

    #[test]
    fn host_paths_preserve_relative_and_explicit_prefix_spelling() {
        let root = tempfile::tempdir().unwrap();
        std::fs::create_dir(root.path().join("child")).unwrap();
        std::fs::write(root.path().join("sample"), "").unwrap();
        std::fs::create_dir(root.path().join("sample-dir")).unwrap();
        let cases: [(PathBuf, String, String, PathBuf, String); 6] = [
            (
                root.path().to_path_buf(),
                "sam".into(),
                "sam".into(),
                PathBuf::from("sample"),
                "sample".into(),
            ),
            (
                root.path().to_path_buf(),
                "./sam".into(),
                "./sam".into(),
                PathBuf::from("./sample"),
                "./sample".into(),
            ),
            (
                root.path().join("child"),
                "../sam".into(),
                "../sam".into(),
                PathBuf::from("../sample"),
                "../sample".into(),
            ),
            (
                root.path().to_path_buf(),
                "child//../sam".into(),
                "child//../sam".into(),
                PathBuf::from("child//../sample"),
                "child//../sample".into(),
            ),
            (
                root.path().to_path_buf(),
                root.path().join("sam").to_str().unwrap().into(),
                root.path().join("sam").to_str().unwrap().into(),
                root.path().join("sample"),
                root.path().join("sample").to_str().unwrap().into(),
            ),
            (
                root.path().to_path_buf(),
                root.path().join("sam").to_str().unwrap().into(),
                "~/sam".into(),
                root.path().join("sample"),
                "~/sample".into(),
            ),
        ];
        for (cwd, lookup_prefix, display_prefix, expected_path, expected_text) in cases {
            let query = FilesystemQuery {
                cwd,
                path_prefix: lookup_prefix,
                insertion_prefix: display_prefix,
                ignore_case: false,
            };
            let candidates = query.collect().unwrap();
            let file = candidates.iter().find(|c| c.kind == "file").unwrap();
            assert_eq!(
                file.literal_path.as_ref().unwrap().as_os_str(),
                expected_path.as_os_str()
            );
            assert_eq!(file.text, expected_text);
            let directory = candidates.iter().find(|c| c.kind == "directory").unwrap();
            assert!(directory.text.ends_with(MAIN_SEPARATOR));
            assert!(
                !directory
                    .literal_path
                    .as_ref()
                    .unwrap()
                    .to_str()
                    .unwrap()
                    .ends_with(is_separator)
            );
        }
        // Prefix splitting must retain a typed trailing separator rather than
        // treating the directory itself as a basename to complete.
        let query = FilesystemQuery {
            cwd: root.path().to_path_buf(),
            path_prefix: "./".into(),
            insertion_prefix: "./".into(),
            ignore_case: false,
        };
        assert!(query.collect().unwrap().iter().any(|c| c.text == "./sample"
            && c.literal_path.as_ref().unwrap().as_os_str() == std::ffi::OsStr::new("./sample")));
        assert_eq!(split_prefix("../child//"), ("../child//", ""));
    }

    #[test]
    fn collection_uses_host_separators_and_request_cwd() {
        let first = tempfile::tempdir().unwrap();
        let second = tempfile::tempdir().unwrap();
        std::fs::create_dir(first.path().join("directory")).unwrap();
        std::fs::write(first.path().join("sample"), "").unwrap();
        let mut query = FilesystemQuery {
            cwd: first.path().into(),
            path_prefix: "sam".into(),
            insertion_prefix: "sam".into(),
            ignore_case: false,
        };
        assert_eq!(
            query.collect().unwrap()[0].literal_path.as_deref(),
            Some(Path::new("sample"))
        );
        query.cwd = second.path().into();
        assert!(query.collect().unwrap().is_empty());
        query.cwd = first.path().into();
        query.path_prefix = "dire".into();
        query.insertion_prefix = "dire".into();
        let rows = query.collect().unwrap();
        assert_eq!(rows[0].text, format!("directory{MAIN_SEPARATOR}"));
        assert_eq!(
            rows[0].literal_path.as_deref(),
            Some(Path::new("directory"))
        );
        let host_prefix = format!("child{MAIN_SEPARATOR}");
        assert_eq!(split_prefix(&host_prefix), (host_prefix.as_str(), ""));
    }

    #[test]
    fn lookup_prefix_policy_does_not_depend_on_filesystem_contents() {
        assert!(supports_lookup_prefix(Path::new("child")));
        assert!(supports_lookup_prefix(Path::new("./child")));
        if is_separator('\\') {
            assert!(!supports_lookup_prefix(Path::new("C:child")));
            assert!(!supports_lookup_prefix(Path::new(r"C:child\sam")));
            assert!(supports_lookup_prefix(Path::new(r"C:\child")));
        } else {
            // On hosts without drive syntax this is an ordinary relative name.
            assert!(supports_lookup_prefix(Path::new("C:child")));
        }
    }

    #[cfg(unix)]
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
            ignore_case: false,
        };
        let rows = query.collect().unwrap();
        let displays: std::collections::HashSet<_> = rows.iter().map(|c| c.text.as_str()).collect();
        assert_eq!(displays.len(), 4);
        assert!(
            rows.iter().any(|c| c.text == "tab\\tname"
                && c.literal_path.as_deref() == Some(Path::new("tab\tname")))
        );
        assert!(rows.iter().any(|c| c.text == "tab\\\\tname"
            && c.literal_path.as_deref() == Some(Path::new("tab\\tname"))));
    }

    #[cfg(unix)]
    #[test]
    fn non_utf8_names_are_skipped_without_lossy_substitution() {
        use std::os::unix::ffi::OsStringExt;
        let temp = tempfile::tempdir().unwrap();
        match std::fs::write(
            temp.path()
                .join(std::ffi::OsString::from_vec(vec![b'a', 0xff])),
            "",
        ) {
            Ok(()) => {}
            Err(error)
                if cfg!(target_os = "macos") && error.raw_os_error() == Some(libc::EILSEQ) =>
            {
                // Some macOS filesystems reject this name before it can be collected.
                eprintln!("filesystem rejects the non-UTF-8 fixture: {error}");
                return;
            }
            Err(error) => panic!("failed to create non-UTF-8 fixture: {error}"),
        }
        let query = FilesystemQuery {
            cwd: temp.path().into(),
            path_prefix: String::new(),
            insertion_prefix: String::new(),
            ignore_case: false,
        };
        assert!(query.collect().unwrap().is_empty());
    }

    #[cfg(unix)]
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
            ignore_case: false,
        };
        let rows = query.collect().unwrap();
        assert_eq!(rows.len(), 5);
        assert!(rows.iter().any(
            |c| c.literal_path.as_deref() == Some(Path::new("line\nname"))
                && c.text == "line\\nname"
        ));
        assert!(
            rows.iter()
                .any(|c| c.literal_path.as_deref() == Some(Path::new("link"))
                    && c.kind == "directory")
        );
        assert!(!rows.iter().any(|c| c.text.starts_with('.')));
    }
    #[cfg(unix)]
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
            ignore_case: false,
        };
        assert_eq!(query.collect().unwrap()[0].text, "a*b");
        query.cwd = second.path().into();
        assert!(query.collect().unwrap().is_empty());
    }
}
