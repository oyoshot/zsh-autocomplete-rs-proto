use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "zsh-autocomplete-rs")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Subcommand)]
pub enum Command {
    /// Read candidates from stdin and show popup completion
    Complete {
        #[arg(long, default_value = "", allow_hyphen_values = true)]
        prefix: String,

        #[arg(long, default_value_t = 0)]
        cursor_row: u16,

        #[arg(long, default_value_t = 0)]
        cursor_col: u16,
    },
}
