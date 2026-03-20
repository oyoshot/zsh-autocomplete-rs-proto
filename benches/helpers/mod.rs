use zsh_autocomplete_rs::candidate::Candidate;

const COMMAND_NAMES: &[&str] = &[
    "git", "git-add", "git-branch", "git-checkout", "git-clone",
    "git-commit", "git-diff", "git-fetch", "git-log", "git-merge",
    "git-pull", "git-push", "git-rebase", "git-reset", "git-stash",
    "git-status", "git-tag", "gitk", "git-bisect", "git-remote",
    "cargo", "cargo-build", "cargo-check", "cargo-clippy", "cargo-doc",
    "cargo-fmt", "cargo-install", "cargo-new", "cargo-run", "cargo-test",
    "cargo-bench", "cargo-update", "cargo-publish", "cargo-add",
    "rustup", "rustc", "rustfmt", "rustdoc", "rust-analyzer",
    "ls", "cat", "grep", "find", "sed", "awk", "sort", "uniq",
    "head", "tail", "wc", "cut", "tr", "tee", "xargs", "chmod",
    "chown", "mkdir", "rmdir", "cp", "mv", "rm", "ln", "touch",
    "docker", "docker-compose", "docker-build", "docker-run",
    "docker-push", "docker-pull", "docker-exec", "docker-logs",
    "npm", "npx", "node", "yarn", "pnpm", "bun",
    "python", "python3", "pip", "pip3", "pytest",
    "make", "cmake", "gcc", "g++", "clang",
    "curl", "wget", "ssh", "scp", "rsync",
];

pub fn generate_candidates(count: usize) -> Vec<Candidate> {
    let mut candidates = Vec::with_capacity(count);
    for i in 0..count {
        let base = COMMAND_NAMES[i % COMMAND_NAMES.len()];
        let (text, description, kind) = if i < COMMAND_NAMES.len() {
            (base.to_string(), format!("{} command", base), "command".to_string())
        } else {
            let suffix = i / COMMAND_NAMES.len();
            (format!("{}-{}", base, suffix), format!("{} variant {}", base, suffix), "command".to_string())
        };
        candidates.push(Candidate { text, description, kind });
    }
    candidates
}

pub fn generate_prefixed_candidates(prefix: &str, count: usize) -> Vec<Candidate> {
    let mut candidates = Vec::with_capacity(count);
    for i in 0..count {
        let base = COMMAND_NAMES[i % COMMAND_NAMES.len()];
        let text = format!("{}{}-{}", prefix, base, i);
        candidates.push(Candidate {
            text,
            description: String::new(),
            kind: "command".to_string(),
        });
    }
    candidates
}
