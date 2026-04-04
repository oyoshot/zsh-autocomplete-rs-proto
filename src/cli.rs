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
            "--prev-popup-row",
            "6",
            "--prev-popup-height",
            "12",
        ]);

        match cli.command {
            Command::Complete {
                shift_tab_hex,
                prev_popup_row,
                prev_popup_height,
                ..
            } => {
                assert_eq!(shift_tab_hex.as_deref(), Some("1b5b32373b323b397e"));
                assert_eq!(prev_popup_row, Some(6));
                assert_eq!(prev_popup_height, Some(12));
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
