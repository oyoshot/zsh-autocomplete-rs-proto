//! Host-native filesystem candidate collection.
//! Shell syntax and display spelling remain adapter-supplied facts.
use crate::candidate::{Candidate, LiteralPath};
use std::path::{Component, MAIN_SEPARATOR, Path, PathBuf, is_separator};
use std::{fs, io};
use unicode_casefold::{Locale, UnicodeCaseFold, Variant};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FilesystemQuery {
    pub cwd: PathBuf,
    /// Literal path used for lookup; any shell expansion has already been verified.
    pub path_prefix: String,
    /// Literal spelling used for candidates (may differ after home expansion).
    pub insertion_prefix: String,
    pub options: FilesystemOptions,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct FilesystemOptions {
    pub ignore_case: bool,
    pub ignore_parent_case: bool,
    pub include_hidden: bool,
    /// Resolve relative to the supplied cwd and insert the absolute result.
    pub absolute_insertion: bool,
}

const MAX_SNAPSHOT_BYTES: usize = 1_048_576;
const MAX_SCANNED_ENTRIES: usize = 65_536;

#[derive(Debug)]
pub enum CollectionError {
    LimitExceeded,
    ReadDirectory(io::Error),
}

impl FilesystemQuery {
    /// Collect an optional query; a request without one has no file candidates.
    pub fn collect_optional(query: Option<Self>) -> Result<Vec<Candidate>, CollectionError> {
        query.map_or_else(|| Ok(Vec::new()), |query| query.collect())
    }

    pub fn collect(&self) -> Result<Vec<Candidate>, CollectionError> {
        self.collect_with_budget(MAX_SNAPSHOT_BYTES, MAX_SCANNED_ENTRIES)
    }

    fn collect_with_budget(
        &self,
        budget: usize,
        remaining_entries: usize,
    ) -> Result<Vec<Candidate>, CollectionError> {
        self.collect_from(budget, remaining_entries, &|path| fs::read_dir(path))
    }

    fn collect_from<I: Iterator<Item = io::Result<fs::DirEntry>>>(
        &self,
        budget: usize,
        mut remaining_entries: usize,
        read_directory: &impl Fn(&Path) -> io::Result<I>,
    ) -> Result<Vec<Candidate>, CollectionError> {
        if !self.cwd.is_absolute()
            || self.path_prefix.contains('\0')
            || self.insertion_prefix.contains('\0')
        {
            return Ok(Vec::new());
        }
        let prefix = Path::new(&self.path_prefix);
        if !supports_lookup_prefix(prefix)
            || (self.options.absolute_insertion
                && matches!(
                    prefix.components().next(),
                    Some(Component::Prefix(_) | Component::RootDir)
                ))
        {
            return Ok(Vec::new());
        }
        let (typed_directory, basename) = split_prefix(&self.path_prefix);
        let directory = Path::new(typed_directory);
        let (insertion_directory, _) = split_prefix(&self.insertion_prefix);
        let (exact_directory, resolved_directories) = if self.options.ignore_parent_case {
            (
                None,
                self.resolve_directories(
                    directory,
                    budget,
                    &mut remaining_entries,
                    read_directory,
                )?,
            )
        } else {
            (Some(directory), Vec::new())
        };
        let display_anchor = if self.options.ignore_parent_case {
            let Some(anchor) = self.insertion_prefix.strip_suffix(&self.path_prefix) else {
                return Ok(Vec::new());
            };
            anchor
        } else {
            ""
        };
        let folded_prefix: Vec<_> = if self.options.ignore_case {
            fold(basename).collect()
        } else {
            Vec::new()
        };
        let mut candidates = Vec::new();
        let mut retained_bytes = 0;
        for directory in exact_directory
            .into_iter()
            .chain(resolved_directories.iter().map(PathBuf::as_path))
        {
            let corrected_display;
            let insertion_directory = if self.options.ignore_parent_case {
                let Some(spelling) = corrected_directory_spelling(typed_directory, directory)
                else {
                    continue;
                };
                corrected_display = format!("{display_anchor}{spelling}");
                &corrected_display
            } else {
                insertion_directory
            };
            let Some(entries) = open_directory(read_directory, &self.cwd.join(directory))? else {
                continue;
            };
            let mut display_directory = String::with_capacity(insertion_directory.len());
            push_display(&mut display_directory, insertion_directory);
            for entry in entries {
                remaining_entries = remaining_entries
                    .checked_sub(1)
                    .ok_or(CollectionError::LimitExceeded)?;
                let entry = entry.map_err(CollectionError::ReadDirectory)?;
                // The engine uses UTF-8. Never replace invalid bytes with a different filename.
                let name = entry.file_name();
                let Some(name_text) = name.to_str() else {
                    continue;
                };
                if !self.options.include_hidden
                    && name_text.starts_with('.')
                    && !basename.starts_with('.')
                {
                    continue;
                }
                if !name_text.starts_with(basename)
                    && !(self.options.ignore_case && {
                        let mut folded_name = fold(name_text);
                        folded_prefix.iter().all(|&c| folded_name.next() == Some(c))
                    })
                {
                    continue;
                }
                let is_directory = is_directory(&entry);
                let path = directory.join(&name);
                let path = if self.options.absolute_insertion {
                    self.cwd.join(path)
                } else {
                    path
                };
                // Size the text as before so the retained-byte budget is unchanged.
                let mut text = String::with_capacity(
                    insertion_directory.len() + name_text.len() + usize::from(is_directory),
                );
                text.push_str(&display_directory);
                push_display(&mut text, name_text);
                if is_directory {
                    // Presentation keeps its separator separate from the actual path.
                    text.push(MAIN_SEPARATOR);
                }
                let path_bytes = path.capacity();
                let candidate = Candidate {
                    text,
                    description: if is_directory { "directory" } else { "file" }.into(),
                    kind: if is_directory { "directory" } else { "file" }.into(),
                    insert_text: None,
                    cursor_offset: None,
                    literal_path: Some(LiteralPath {
                        path,
                        ignore_case: self.options.ignore_case,
                    }),
                };
                // Count owned data as well as row storage. Abandon the whole query
                // on overflow, since a partial snapshot could be auto-selected.
                retained_bytes += size_of::<Candidate>()
                    + candidate.text.capacity()
                    + candidate.description.capacity()
                    + candidate.kind.capacity()
                    + path_bytes;
                if retained_bytes > budget {
                    return Err(CollectionError::LimitExceeded);
                }
                candidates.push(candidate);
            }
        }
        candidates.sort_by(|a, b| a.text.cmp(&b.text));
        Ok(candidates)
    }

    fn resolve_directories<I: Iterator<Item = io::Result<fs::DirEntry>>>(
        &self,
        directory: &Path,
        budget: usize,
        remaining_entries: &mut usize,
        read_directory: &impl Fn(&Path) -> io::Result<I>,
    ) -> Result<Vec<PathBuf>, CollectionError> {
        let mut directories = vec![PathBuf::new()];
        for component in directory.components() {
            let Component::Normal(name) = component else {
                let mut remaining_bytes = budget;
                for path in &mut directories {
                    path.push(component.as_os_str());
                    remaining_bytes = remaining_bytes
                        .checked_sub(size_of::<PathBuf>() + path.capacity())
                        .ok_or(CollectionError::LimitExceeded)?;
                }
                continue;
            };
            let Some(name) = name.to_str() else {
                return Ok(Vec::new());
            };
            let folded: Vec<_> = fold(name).collect();
            let mut matches = Vec::new();
            let mut remaining_bytes = budget;
            for path in directories {
                let Some(entries) = open_directory(read_directory, &self.cwd.join(&path))? else {
                    continue;
                };
                for entry in entries {
                    *remaining_entries = remaining_entries
                        .checked_sub(1)
                        .ok_or(CollectionError::LimitExceeded)?;
                    let entry = entry.map_err(CollectionError::ReadDirectory)?;
                    let name = entry.file_name();
                    if name
                        .to_str()
                        .is_some_and(|text| fold(text).eq(folded.iter().copied()))
                        && is_directory(&entry)
                    {
                        let next = path.join(name);
                        remaining_bytes = remaining_bytes
                            .checked_sub(size_of::<PathBuf>() + next.capacity())
                            .ok_or(CollectionError::LimitExceeded)?;
                        matches.push(next);
                    }
                }
            }
            if matches.is_empty() {
                return Ok(matches);
            }
            directories = matches;
        }
        Ok(directories)
    }
}

/// A directory, or a symlink that resolves to one.
fn is_directory(entry: &fs::DirEntry) -> bool {
    entry
        .file_type()
        .is_ok_and(|kind| kind.is_dir() || (kind.is_symlink() && entry.path().is_dir()))
}

/// Append display text: controls and literal backslashes get distinct escapes.
fn push_display(text: &mut String, raw: &str) {
    for c in raw.chars() {
        if c == '\\' && !is_separator(c) {
            text.push_str("\\\\");
        } else if c.is_control() {
            text.extend(c.escape_default());
        } else {
            text.push(c);
        }
    }
}

fn open_directory<I>(
    read_directory: &impl Fn(&Path) -> io::Result<I>,
    path: &Path,
) -> Result<Option<I>, CollectionError> {
    match read_directory(path) {
        Ok(entries) => Ok(Some(entries)),
        Err(error)
            if matches!(
                error.kind(),
                io::ErrorKind::NotFound | io::ErrorKind::NotADirectory
            ) =>
        {
            Ok(None)
        }
        Err(error) => Err(CollectionError::ReadDirectory(error)),
    }
}

fn fold(text: &str) -> impl Iterator<Item = char> + '_ {
    text.case_fold_with(Variant::Simple, Locale::NonTurkic)
}

/// Substitute the on-disk names without normalizing the typed separators or dots.
fn corrected_directory_spelling(typed: &str, resolved: &Path) -> Option<String> {
    let prefix_len = match Path::new(typed).components().next() {
        Some(Component::Prefix(prefix)) => prefix.as_os_str().len(),
        _ => 0,
    };
    let (prefix, rest) = typed.split_at(prefix_len);
    let mut names = resolved
        .components()
        .filter_map(|component| match component {
            Component::Normal(name) => Some(name),
            _ => None,
        });
    let mut spelling = String::with_capacity(typed.len());
    spelling.push_str(prefix);
    for part in rest.split_inclusive(is_separator) {
        let (typed_name, separator) = part.split_at(part.trim_end_matches(is_separator).len());
        let name = match typed_name {
            "" | "." | ".." => typed_name,
            _ => names.next()?.to_str()?,
        };
        spelling.push_str(name);
        spelling.push_str(separator);
    }
    names.next().is_none().then_some(spelling)
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
            options: FilesystemOptions::default(),
        };
        assert!(query.collect().unwrap().is_empty());
        query.options.ignore_case = true;
        let candidates = query.collect().unwrap();
        assert_eq!(candidates.len(), 1);
        assert_eq!(candidates[0].text, "Ωmega");
        assert_eq!(
            candidates[0].literal_path.as_deref(),
            Some(Path::new("Ωmega"))
        );
    }

    #[test]
    fn folded_parent_lookup_preserves_actual_names_under_the_query_anchor() {
        let root = tempfile::tempdir().unwrap();
        let actual = root.path().join("Folder").join("Child");
        std::fs::create_dir_all(&actual).unwrap();
        std::fs::write(actual.join("Name"), "").unwrap();
        std::fs::write(actual.join(".hidden"), "").unwrap();
        let typed = format!("folder{MAIN_SEPARATOR}CHILD{MAIN_SEPARATOR}");
        let mut query = FilesystemQuery {
            cwd: root.path().into(),
            path_prefix: format!("{typed}na"),
            insertion_prefix: format!("~/{typed}na"),
            options: FilesystemOptions {
                ignore_case: true,
                ignore_parent_case: true,
                absolute_insertion: true,
                ..FilesystemOptions::default()
            },
        };
        let rows = query.collect().unwrap();
        assert_eq!(rows.len(), 1);
        assert_eq!(
            rows[0].text,
            format!("~/Folder{MAIN_SEPARATOR}Child{MAIN_SEPARATOR}Name")
        );
        assert_eq!(
            rows[0].literal_path.as_deref(),
            Some(actual.join("Name").as_path())
        );

        query.path_prefix = typed.clone();
        query.insertion_prefix = format!("~/{typed}");
        assert_eq!(query.collect().unwrap().len(), 1);
        query.options.include_hidden = true;
        assert_eq!(query.collect().unwrap().len(), 2);
    }

    #[test]
    fn oversized_snapshot_is_rejected_as_a_whole() {
        let root = tempfile::tempdir().unwrap();
        let query = FilesystemQuery {
            cwd: root.path().into(),
            path_prefix: String::new(),
            insertion_prefix: String::new(),
            options: FilesystemOptions::default(),
        };
        let budget = size_of::<Candidate>() + 64;
        std::fs::write(root.path().join("first"), "").unwrap();
        assert_eq!(
            query
                .collect_with_budget(budget, MAX_SCANNED_ENTRIES)
                .unwrap()
                .len(),
            1
        );
        std::fs::write(root.path().join("other"), "").unwrap();
        assert!(matches!(
            query.collect_with_budget(budget, MAX_SCANNED_ENTRIES),
            Err(CollectionError::LimitExceeded)
        ));
    }

    #[test]
    fn snapshot_budget_admits_exactly_the_bytes_the_candidates_retain() {
        let root = tempfile::tempdir().unwrap();
        std::fs::write(root.path().join("file"), "").unwrap();
        std::fs::write(root.path().join("longer-file-name"), "").unwrap();
        std::fs::create_dir(root.path().join("directory")).unwrap();
        for absolute_insertion in [false, true] {
            let query = FilesystemQuery {
                cwd: root.path().into(),
                path_prefix: String::new(),
                insertion_prefix: String::new(),
                options: FilesystemOptions {
                    absolute_insertion,
                    ..FilesystemOptions::default()
                },
            };
            let retained: usize = query
                .collect()
                .unwrap()
                .iter()
                .map(|candidate| {
                    size_of::<Candidate>()
                        + candidate.text.capacity()
                        + candidate.description.capacity()
                        + candidate.kind.capacity()
                        + candidate.literal_path.as_ref().unwrap().path.capacity()
                })
                .sum();

            assert_eq!(
                query
                    .collect_with_budget(retained, MAX_SCANNED_ENTRIES)
                    .unwrap()
                    .len(),
                3
            );
            assert!(matches!(
                query.collect_with_budget(retained - 1, MAX_SCANNED_ENTRIES),
                Err(CollectionError::LimitExceeded)
            ));
        }
    }

    #[test]
    fn scan_limit_counts_nonmatching_entries_across_parent_and_leaf() {
        let root = tempfile::tempdir().unwrap();
        let parent = root.path().join("Folder");
        std::fs::create_dir(&parent).unwrap();
        std::fs::write(root.path().join("unrelated"), "").unwrap();
        std::fs::write(parent.join("first"), "").unwrap();
        std::fs::write(parent.join("second"), "").unwrap();
        let mut query = FilesystemQuery {
            cwd: root.path().into(),
            path_prefix: "absent".into(),
            insertion_prefix: "absent".into(),
            options: FilesystemOptions::default(),
        };
        assert!(query.collect_with_budget(MAX_SNAPSHOT_BYTES, 1).is_err());
        assert!(
            query
                .collect_with_budget(MAX_SNAPSHOT_BYTES, 2)
                .unwrap()
                .is_empty()
        );
        query.path_prefix = format!("folder{MAIN_SEPARATOR}absent");
        query.insertion_prefix = query.path_prefix.clone();
        query.options.ignore_parent_case = true;
        assert!(query.collect_with_budget(MAX_SNAPSHOT_BYTES, 3).is_err());
        assert!(
            query
                .collect_with_budget(MAX_SNAPSHOT_BYTES, 4)
                .unwrap()
                .is_empty()
        );
    }

    #[test]
    fn a_directory_iteration_error_discards_the_snapshot() {
        let root = tempfile::tempdir().unwrap();
        let parent = root.path().join("Folder");
        fs::create_dir(&parent).unwrap();
        fs::write(parent.join("match"), "").unwrap();
        let typed_path = format!("folder{MAIN_SEPARATOR}mat");
        for (cwd, prefix, ignore_parent_case, failing_directory) in [
            (parent.as_path(), "mat", false, parent.as_path()),
            (root.path(), typed_path.as_str(), true, root.path()),
            (root.path(), typed_path.as_str(), true, parent.as_path()),
        ] {
            let query = FilesystemQuery {
                cwd: cwd.into(),
                path_prefix: prefix.into(),
                insertion_prefix: prefix.into(),
                options: FilesystemOptions {
                    ignore_parent_case,
                    ..FilesystemOptions::default()
                },
            };
            assert_eq!(query.collect().unwrap().len(), 1);
            let read_directory = |path: &Path| {
                let error = (path == failing_directory)
                    .then(|| Err(io::Error::from(io::ErrorKind::ConnectionReset)));
                Ok(fs::read_dir(path)?.chain(error))
            };
            assert!(matches!(
                query.collect_from(MAX_SNAPSHOT_BYTES, MAX_SCANNED_ENTRIES, &read_directory),
                Err(CollectionError::ReadDirectory(error)) if error.kind() == io::ErrorKind::ConnectionReset
            ));
        }
    }

    #[test]
    fn appending_parents_counts_against_the_directory_budget() {
        let root = tempfile::tempdir().unwrap();
        fs::create_dir(root.path().join("Folder")).unwrap();
        let query = FilesystemQuery {
            cwd: root.path().into(),
            path_prefix: "folder".into(),
            insertion_prefix: "folder".into(),
            options: FilesystemOptions::default(),
        };
        let budget = size_of::<PathBuf>() + PathBuf::new().join("Folder").capacity();
        let mut parents = PathBuf::from("folder");
        for _ in 0..8 {
            parents.push("..");
        }
        let read_directory = |path: &Path| fs::read_dir(path);
        assert!(
            query
                .resolve_directories(Path::new("folder"), budget, &mut 16, &read_directory)
                .is_ok()
        );
        assert!(matches!(
            query.resolve_directories(&parents, budget, &mut 16, &read_directory),
            Err(CollectionError::LimitExceeded)
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
                options: FilesystemOptions::default(),
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
            options: FilesystemOptions::default(),
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
            options: FilesystemOptions::default(),
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
            options: FilesystemOptions::default(),
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

    /// Reverses `push_display` on hosts where a backslash is not a separator.
    #[cfg(unix)]
    fn parse_display(display: &str) -> Option<String> {
        let mut raw = String::new();
        let mut chars = display.chars();
        while let Some(c) = chars.next() {
            if c != '\\' {
                raw.push(c);
                continue;
            }
            raw.push(match chars.next()? {
                '\\' => '\\',
                't' => '\t',
                'n' => '\n',
                'r' => '\r',
                'u' => {
                    let rest = chars.as_str().strip_prefix('{')?;
                    let (digits, rest) = rest.split_once('}')?;
                    chars = rest.chars();
                    char::from_u32(u32::from_str_radix(digits, 16).ok()?)?
                }
                _ => return None,
            });
        }
        Some(raw)
    }

    /// Randomized only. Distinct names never share a display, because every
    /// display parses back to the one name that produced it.
    #[cfg(unix)]
    #[test]
    fn display_is_control_free_and_reversible_for_any_name() {
        bolero::check!().with_type::<String>().for_each(|raw| {
            let mut display = String::new();
            push_display(&mut display, raw);
            assert!(!display.chars().any(char::is_control));
            assert_eq!(parse_display(&display).as_ref(), Some(raw));
        });
    }

    /// Randomized only.
    #[test]
    fn split_prefix_cuts_after_the_last_separator_for_any_prefix() {
        bolero::check!().with_type::<String>().for_each(|prefix| {
            let (directory, basename) = split_prefix(prefix);
            assert_eq!(format!("{directory}{basename}"), *prefix);
            assert!(!basename.contains(is_separator));
            assert!(directory.is_empty() || directory.ends_with(is_separator));
        });
    }

    /// Randomized only. A spelling that already names the resolved directories
    /// comes back unchanged, including its dots and repeated separators.
    #[cfg(unix)]
    #[test]
    fn corrected_spelling_keeps_any_typed_path_that_already_matches() {
        const PARTS: [&str; 6] = ["", ".", "..", "name", "Name", "\u{e9}t\u{e9}"];
        bolero::check!().with_type::<Vec<u8>>().for_each(|choices| {
            let typed = choices
                .iter()
                .map(|&choice| PARTS[usize::from(choice) % PARTS.len()])
                .collect::<Vec<_>>()
                .join("/");
            assert_eq!(
                corrected_directory_spelling(&typed, Path::new(&typed)),
                Some(typed)
            );
        });
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
            options: FilesystemOptions::default(),
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
            options: FilesystemOptions::default(),
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
            options: FilesystemOptions::default(),
        };
        assert_eq!(query.collect().unwrap()[0].text, "a*b");
        query.cwd = second.path().into();
        assert!(query.collect().unwrap().is_empty());
    }
}
