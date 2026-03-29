use zsh_autocomplete_rs::app::App;
use zsh_autocomplete_rs::candidate::Candidate;
use zsh_autocomplete_rs::cli::{Cli, Command, DaemonAction};
use zsh_autocomplete_rs::handoff::compute_reuse_token;
use zsh_autocomplete_rs::{client, config, daemon, protocol, tty, ui};

use clap::Parser;
use std::io::{self, BufWriter, Read, Write};
use std::process;

fn run_complete(
    prefix: String,
    cursor_row: u16,
    cursor_col: u16,
    cols: u16,
    rows: u16,
    shift_tab_sequence: Option<Vec<u8>>,
) -> io::Result<()> {
    let stdin = io::stdin();
    let mut reader = io::BufReader::new(stdin.lock());
    let tsv = daemon::read_tsv(&mut reader).map_err(io::Error::other)?;

    let stdout = io::stdout();
    let mut writer = BufWriter::new(stdout.lock());

    daemon::run_stdio_complete(
        &mut reader,
        &mut writer,
        prefix,
        cursor_row,
        cursor_col,
        cols,
        rows,
        shift_tab_sequence,
        &tsv,
    );
    Ok(())
}

fn run_render(
    prefix: String,
    cursor_row: u16,
    cursor_col: u16,
    selected: Option<usize>,
    theme: &config::Theme,
) -> io::Result<i32> {
    // Read raw stdin before trying daemon (we need it for both paths)
    let raw_stdin: Vec<u8> = {
        let mut buf = Vec::new();
        io::stdin().lock().read_to_end(&mut buf)?;
        buf
    };

    // Try daemon first
    if let Ok(resp) =
        client::try_daemon_render(&prefix, cursor_row, cursor_col, selected, &raw_stdin)
    {
        let mut tty = tty::open_tty_write()?;
        tty.write_all(&resp.tty_bytes)?;
        tty.flush()?;
        if let Some(meta) = resp.metadata {
            println!("{}", meta);
        }
        return Ok(0);
    }

    // Fallback: direct execution
    let candidates: Vec<Candidate> = std::str::from_utf8(&raw_stdin)
        .unwrap_or("")
        .lines()
        .filter(|line| !line.is_empty())
        .map(Candidate::parse_line)
        .collect();

    if candidates.is_empty() {
        return Ok(1);
    }

    let mut app = App::new(candidates, prefix, cursor_row, cursor_col);
    if app.filtered_indices.is_empty() {
        return Ok(1);
    }

    let mut tty = tty::open_tty_write()?;
    ui::render::ensure_space(&mut tty, &mut app)?;

    if app.max_visible == 0 {
        return Ok(1);
    }

    if let Some(idx) = selected {
        app.set_selected(idx);
    }

    ui::render::draw_popup_only(&mut tty, &app, theme)?;

    let popup = ui::popup::Popup::compute(&app);
    let candidates_tsv = std::str::from_utf8(&raw_stdin).unwrap_or("");
    let reuse_token = compute_reuse_token(&app.prefix, candidates_tsv, &app, &popup);
    let meta = popup.format_metadata(
        app.cursor_row,
        reuse_token,
        app.filtered_indices.len(),
        app.selected_original_idx(),
    );
    println!("{}", meta);

    Ok(0)
}

fn run_clear(popup_row: u16, popup_height: u16, cursor_row: u16) -> io::Result<i32> {
    // Try daemon first
    if let Ok(tty_bytes) = client::try_daemon_clear(popup_row, popup_height, cursor_row) {
        if !tty_bytes.is_empty() {
            let mut tty = tty::open_tty_write()?;
            tty.write_all(&tty_bytes)?;
            tty.flush()?;
        }
        return Ok(0);
    }

    // Fallback: direct execution
    let mut tty = tty::open_tty_write()?;
    ui::render::clear_rect(&mut tty, popup_row, popup_height, cursor_row)?;
    Ok(0)
}

fn main() {
    let cli = Cli::parse();
    match cli.command {
        Command::Complete {
            prefix,
            cursor_row,
            cursor_col,
            shift_tab_hex,
            cols,
            rows,
        } => match run_complete(
            prefix,
            cursor_row,
            cursor_col,
            cols,
            rows,
            shift_tab_hex
                .as_deref()
                .and_then(protocol::decode_hex_bytes),
        ) {
            Ok(()) => process::exit(0),
            Err(e) => {
                eprintln!("error: {}", e);
                process::exit(1);
            }
        },
        Command::Render {
            prefix,
            cursor_row,
            cursor_col,
            selected,
        } => {
            let theme = config::Config::load().theme();
            match run_render(prefix, cursor_row, cursor_col, selected, &theme) {
                Ok(code) => process::exit(code),
                Err(e) => {
                    eprintln!("error: {}", e);
                    process::exit(1);
                }
            }
        }
        Command::Clear {
            popup_row,
            popup_height,
            cursor_row,
        } => match run_clear(popup_row, popup_height, cursor_row) {
            Ok(code) => process::exit(code),
            Err(e) => {
                eprintln!("error: {}", e);
                process::exit(1);
            }
        },
        Command::Daemon { action } => match action {
            DaemonAction::Start => match daemon::start() {
                Ok(()) => process::exit(0),
                Err(e) => {
                    eprintln!("daemon error: {}", e);
                    process::exit(1);
                }
            },
            DaemonAction::Stop => match daemon::stop() {
                Ok(()) => process::exit(0),
                Err(e) => {
                    eprintln!("daemon stop error: {}", e);
                    process::exit(1);
                }
            },
            DaemonAction::Status => {
                if daemon::status() {
                    println!("running");
                    process::exit(0);
                } else {
                    println!("stopped");
                    process::exit(1);
                }
            }
        },
    }
}
