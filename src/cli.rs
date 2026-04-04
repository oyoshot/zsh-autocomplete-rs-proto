use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "zsh-autocomplete-rs")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Subcommand)]
pub enum Command {
    /// Popup session — Tab で起動
    Complete {
        #[arg(long, default_value = "", allow_hyphen_values = true)]
        prefix: String,

        #[arg(long, default_value_t = 0)]
        cursor_row: u16,

        #[arg(long, default_value_t = 0)]
        cursor_col: u16,

        #[arg(long)]
        shift_tab_hex: Option<String>,

        #[arg(long, default_value_t = false)]
        daemon: bool,

        #[arg(long, default_value_t = false)]
        command_position: bool,

        #[arg(long)]
        stale_hex: Option<String>,

        #[arg(long)]
        reuse_token: Option<String>,

        #[arg(long)]
        context_key: Option<String>,

        #[arg(long)]
        prev_popup_row: Option<u16>,

        #[arg(long)]
        prev_popup_height: Option<u16>,

        #[arg(long, default_value_t = 80)]
        cols: u16,

        #[arg(long, default_value_t = 24)]
        rows: u16,
    },
    /// Draw popup and exit immediately (non-blocking) — 自動トリガー用
    Render {
        #[arg(long, default_value = "", allow_hyphen_values = true)]
        prefix: String,

        #[arg(long, default_value_t = 0)]
        cursor_row: u16,

        #[arg(long, default_value_t = 0)]
        cursor_col: u16,

        /// Pre-select the N-th filtered candidate (0-indexed)
        #[arg(long)]
        selected: Option<usize>,
    },
    /// Clear a previously rendered popup
    Clear {
        #[arg(long)]
        popup_row: u16,

        #[arg(long)]
        popup_height: u16,

        #[arg(long)]
        cursor_row: u16,
    },
    /// Resolve single-candidate completion text using config-driven suffix rules
    ResolveSingle {
        #[arg(long, allow_hyphen_values = true)]
        text: String,

        #[arg(long, default_value = "")]
        kind: String,

        #[arg(long, default_value_t = false)]
        command_position: bool,
    },
    /// Daemon management
    Daemon {
        #[command(subcommand)]
        action: DaemonAction,
    },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn complete_accepts_shift_tab_hex() {
        let cli = Cli::parse_from([
            "zsh-autocomplete-rs",
            "complete",
            "--shift-tab-hex",
            "1b5b32373b323b397e",
            "--daemon",
            "--command-position",
            "--stale-hex",
            "1b5b44",
            "--reuse-token",
            "123",
            "--context-key",
            "ctx",
            "--prev-popup-row",
            "6",
            "--prev-popup-height",
            "12",
        ]);

        match cli.command {
            Command::Complete {
                shift_tab_hex,
                daemon,
                command_position,
                stale_hex,
                reuse_token,
                context_key,
                prev_popup_row,
                prev_popup_height,
                ..
            } => {
                assert_eq!(shift_tab_hex.as_deref(), Some("1b5b32373b323b397e"));
                assert!(daemon);
                assert!(command_position);
                assert_eq!(stale_hex.as_deref(), Some("1b5b44"));
                assert_eq!(reuse_token.as_deref(), Some("123"));
                assert_eq!(context_key.as_deref(), Some("ctx"));
                assert_eq!(prev_popup_row, Some(6));
                assert_eq!(prev_popup_height, Some(12));
            }
            _ => panic!("unexpected command"),
        }
    }

    #[test]
    fn resolve_single_accepts_hyphenated_text() {
        let cli = Cli::parse_from([
            "zsh-autocomplete-rs",
            "resolve-single",
            "--text",
            "--watch",
            "--kind",
            "command",
            "--command-position",
        ]);

        match cli.command {
            Command::ResolveSingle {
                text,
                kind,
                command_position,
            } => {
                assert_eq!(text, "--watch");
                assert_eq!(kind, "command");
                assert!(command_position);
            }
            _ => panic!("unexpected command"),
        }
    }
}

#[derive(Subcommand)]
pub enum DaemonAction {
    /// Start daemon in foreground
    Start,
    /// Stop running daemon
    Stop,
    /// Check if daemon is running (exit 0 = running, 1 = not)
    Status,
}
