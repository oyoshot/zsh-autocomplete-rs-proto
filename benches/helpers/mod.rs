#![allow(dead_code)]

use zsh_autocomplete_rs::candidate::Candidate;

const COMMAND_NAMES: &[&str] = &[
    "git",
    "git-add",
    "git-branch",
    "git-checkout",
    "git-clone",
    "git-commit",
    "git-diff",
    "git-fetch",
    "git-log",
    "git-merge",
    "git-pull",
    "git-push",
    "git-rebase",
    "git-reset",
    "git-stash",
    "git-status",
    "git-tag",
    "gitk",
    "git-bisect",
    "git-remote",
    "cargo",
    "cargo-build",
    "cargo-check",
    "cargo-clippy",
    "cargo-doc",
    "cargo-fmt",
    "cargo-install",
    "cargo-new",
    "cargo-run",
    "cargo-test",
    "cargo-bench",
    "cargo-update",
    "cargo-publish",
    "cargo-add",
    "rustup",
    "rustc",
    "rustfmt",
    "rustdoc",
    "rust-analyzer",
    "ls",
    "cat",
    "grep",
    "find",
    "sed",
    "awk",
    "sort",
    "uniq",
    "head",
    "tail",
    "wc",
    "cut",
    "tr",
    "tee",
    "xargs",
    "chmod",
    "chown",
    "mkdir",
    "rmdir",
    "cp",
    "mv",
    "rm",
    "ln",
    "touch",
    "docker",
    "docker-compose",
    "docker-build",
    "docker-run",
    "docker-push",
    "docker-pull",
    "docker-exec",
    "docker-logs",
    "npm",
    "npx",
    "node",
    "yarn",
    "pnpm",
    "bun",
    "python",
    "python3",
    "pip",
    "pip3",
    "pytest",
    "make",
    "cmake",
    "gcc",
    "g++",
    "clang",
    "claude",
    "curl",
    "wget",
    "ssh",
    "scp",
    "rsync",
];

const UNICODE_COMMAND_NAMES: &[&str] = &[
    "café",
    "résumé",
    "naïve",
    "jalapeño",
    "São-Paulo",
    "Äac",
    "ångström",
    "élan",
    "über",
    "niño",
    "façade",
    "coöperate",
    "smörgåsbord",
    "crème-brûlée",
    "Łódź",
    "İstanbul",
    "mañana",
    "doppelgänger",
    "fiancée",
    "protégé",
];

pub fn generate_candidates(count: usize) -> Vec<Candidate> {
    let mut candidates = Vec::with_capacity(count);
    for i in 0..count {
        let base = COMMAND_NAMES[i % COMMAND_NAMES.len()];
        let (text, description, kind) = if i < COMMAND_NAMES.len() {
            (
                base.to_string(),
                format!("{} command", base),
                "command".to_string(),
            )
        } else {
            let suffix = i / COMMAND_NAMES.len();
            (
                format!("{}-{}", base, suffix),
                format!("{} variant {}", base, suffix),
                "command".to_string(),
            )
        };
        candidates.push(Candidate {
            text,
            description,
            kind,
        });
    }
    candidates
}

pub fn generate_unicode_candidates(count: usize) -> Vec<Candidate> {
    let mut candidates = Vec::with_capacity(count);
    for i in 0..count {
        let base = UNICODE_COMMAND_NAMES[i % UNICODE_COMMAND_NAMES.len()];
        let (text, description, kind) = if i < UNICODE_COMMAND_NAMES.len() {
            (
                base.to_string(),
                format!("{} command", base),
                "command".to_string(),
            )
        } else {
            let suffix = i / UNICODE_COMMAND_NAMES.len();
            (
                format!("{}-{}", base, suffix),
                format!("{} variant {}", base, suffix),
                "command".to_string(),
            )
        };
        candidates.push(Candidate {
            text,
            description,
            kind,
        });
    }
    candidates
}

pub fn candidates_to_tsv(candidates: &[Candidate]) -> String {
    let mut tsv = String::new();
    for c in candidates {
        tsv.push_str(&c.text);
        tsv.push('\t');
        tsv.push_str(&c.description);
        tsv.push('\t');
        tsv.push_str(&c.kind);
        tsv.push('\n');
    }
    tsv
}

#[allow(dead_code)]
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

const CJK_COMMAND_NAMES: &[&str] = &[
    "git-ブランチ",
    "cargo-ビルド",
    "ファイル一覧",
    "docker-実行",
    "検索コマンド",
    "npm-インストール",
    "設定ファイル",
    "make-ビルド",
    "テスト実行",
    "ssh-接続",
    "ディレクトリ作成",
    "python-スクリプト",
    "パッケージ管理",
    "curl-ダウンロード",
    "ログ表示",
    "rsync-同期",
    "プロセス一覧",
    "gcc-コンパイル",
    "ネットワーク診断",
    "vim-編集",
];

pub fn generate_cjk_candidates(count: usize) -> Vec<Candidate> {
    let mut candidates = Vec::with_capacity(count);
    for i in 0..count {
        let base = CJK_COMMAND_NAMES[i % CJK_COMMAND_NAMES.len()];
        let (text, description, kind) = if i < CJK_COMMAND_NAMES.len() {
            (
                base.to_string(),
                format!("{}の説明", base),
                "command".to_string(),
            )
        } else {
            let suffix = i / CJK_COMMAND_NAMES.len();
            (
                format!("{}-{}", base, suffix),
                format!("{} バリアント {}", base, suffix),
                "command".to_string(),
            )
        };
        candidates.push(Candidate {
            text,
            description,
            kind,
        });
    }
    candidates
}

pub fn generate_no_description_candidates(count: usize) -> Vec<Candidate> {
    let mut candidates = Vec::with_capacity(count);
    for i in 0..count {
        let base = COMMAND_NAMES[i % COMMAND_NAMES.len()];
        let text = if i < COMMAND_NAMES.len() {
            base.to_string()
        } else {
            format!("{}-{}", base, i / COMMAND_NAMES.len())
        };
        candidates.push(Candidate {
            text,
            description: String::new(),
            kind: "command".to_string(),
        });
    }
    candidates
}

pub fn generate_long_description_candidates(count: usize) -> Vec<Candidate> {
    let mut candidates = Vec::with_capacity(count);
    for i in 0..count {
        let base = COMMAND_NAMES[i % COMMAND_NAMES.len()];
        let text = if i < COMMAND_NAMES.len() {
            base.to_string()
        } else {
            format!("{}-{}", base, i / COMMAND_NAMES.len())
        };
        let description = format!(
            "{} - a comprehensive tool for managing complex workflows, \
             including build automation, dependency resolution, and deployment \
             orchestration across multiple environments (variant {})",
            base, i,
        );
        candidates.push(Candidate {
            text,
            description,
            kind: "command".to_string(),
        });
    }
    candidates
}
