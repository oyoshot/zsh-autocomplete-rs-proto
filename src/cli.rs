use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "zsh-autocomplete-rs")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Subcommand)]
pub enum Command {
    /// Interactive popup (blocking) — Tab で起動
    Complete {
        #[arg(long, default_value = "", allow_hyphen_values = true)]
        prefix: String,

        #[arg(long, default_value_t = 0)]
        cursor_row: u16,

        #[arg(long, default_value_t = 0)]
        cursor_col: u16,
    },
    /// Draw popup and exit immediately (non-blocking) — 自動トリガー用
    Render {
        #[arg(long, default_value = "", allow_hyphen_values = true)]
        prefix: String,

        #[arg(long, default_value_t = 0)]
        cursor_row: u16,

        #[arg(long, default_value_t = 0)]
        cursor_col: u16,
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
}
