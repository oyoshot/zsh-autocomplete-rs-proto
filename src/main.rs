use zsh_autocomplete_rs::app::App;
use zsh_autocomplete_rs::candidate::Candidate;
use zsh_autocomplete_rs::cli::{Cli, Command, DaemonAction};
use zsh_autocomplete_rs::handoff::compute_reuse_token;
use zsh_autocomplete_rs::{client, config, daemon, protocol, tty, ui};

use clap::Parser;
use std::io::{self, BufRead, BufWriter, Read, Write};
use std::os::unix::net::UnixStream;
use std::process;
use std::thread;

fn trim_line_end(line: &str) -> &str {
    line.trim_end_matches(['\r', '\n'])
}

fn write_done_result(
    mut writer: impl Write,
    result: &client::CompleteSessionResult,
) -> io::Result<()> {
    writeln!(writer, "DONE {} {}", result.code, result.text)?;
    writeln!(
        writer,
        "APPLY chain={} execute={} restore={}",
        if result.chain { 1 } else { 0 },
        if result.execute { 1 } else { 0 },
        result.restore_text
    )?;
    writer.flush()
}

struct CompleteCommand {
    prefix: String,
    cursor_row: u16,
    cursor_col: u16,
    cols: u16,
    rows: u16,
    daemon_mode: bool,
    shift_tab_sequence: Option<Vec<u8>>,
    stale_bytes: Vec<u8>,
    reuse_token: Option<String>,
    context_key: Option<String>,
    prev_popup_row: Option<u16>,
    prev_popup_height: Option<u16>,
}

fn run_complete(command: CompleteCommand) -> io::Result<()> {
    let CompleteCommand {
        prefix,
        cursor_row,
        cursor_col,
        cols,
        rows,
        daemon_mode,
        shift_tab_sequence,
        stale_bytes,
        reuse_token,
        context_key,
        prev_popup_row,
        prev_popup_height,
    } = command;

    let stdin = io::stdin();
    let mut reader = io::BufReader::new(stdin.lock());
    let tsv = daemon::read_tsv_payload(&mut reader).map_err(io::Error::other)?;

    if daemon_mode {
        let stdout = io::stdout();
        let mut writer = BufWriter::new(stdout.lock());
        match client::try_daemon_complete(
            &prefix,
            cursor_row,
            cursor_col,
            &tsv,
            shift_tab_sequence,
            stale_bytes,
            prev_popup_row.zip(prev_popup_height),
            reuse_token.as_deref(),
            context_key.as_deref(),
        ) {
            Ok(client::CompleteSessionOutcome::Done(result)) => {
                write_done_result(&mut writer, &result)?
            }
            Ok(client::CompleteSessionOutcome::CacheMiss) => {
                writeln!(writer, "CACHE_MISS")?;
                writer.flush()?;
            }
            Err(e) => return Err(io::Error::other(e.to_string())),
        }
        return Ok(());
    }

    let stdout = io::stdout();
    let mut stdout_writer = BufWriter::new(stdout.lock());
    let (server_stream, client_stream) = UnixStream::pair()?;
    let prefix_for_thread = prefix;
    let tsv_for_thread = tsv;
    let shift_tab_for_thread = shift_tab_sequence;
    let handle = thread::spawn(move || {
        let mut session_reader = io::BufReader::new(&server_stream);
        let mut session_writer = io::BufWriter::new(&server_stream);
        daemon::run_stdio_complete(
            &mut session_reader,
            &mut session_writer,
            prefix_for_thread,
            cursor_row,
            cursor_col,
            cols,
            rows,
            shift_tab_for_thread,
            prev_popup_row,
            prev_popup_height,
            &tsv_for_thread,
        );
    });
    let mut session_reader = io::BufReader::new(client_stream.try_clone()?);
    let mut session_writer = client_stream;
    let mut initial_header = String::new();
    session_reader.read_line(&mut initial_header)?;
    let result = client::run_text_popup_session(
        &mut session_reader,
        &mut session_writer,
        trim_line_end(&initial_header),
        stale_bytes,
    )
    .map_err(|e| io::Error::other(e.to_string()))?;
    write_done_result(&mut stdout_writer, &result)?;
    handle
        .join()
        .map_err(|_| io::Error::other("complete session thread panicked"))?;
    Ok(())
}

fn run_render(
    prefix: String,
    cursor_row: u16,
    cursor_col: u16,
    selected: Option<usize>,
    theme: &config::Theme,
    auto_insert_unambiguous: bool,
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

    if !auto_insert_unambiguous {
        app.reset_filter_to_prefix();
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
    // Render metadata is for popup position only; auto-insert is handled
    // exclusively via FRAME headers in the complete (interactive) path.
    let meta = popup.format_metadata(
        app.cursor_row,
        reuse_token,
        app.filtered_indices.len(),
        app.selected_original_idx(),
        None,
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
            daemon,
            shift_tab_hex,
            stale_hex,
            reuse_token,
            context_key,
            prev_popup_row,
            prev_popup_height,
            cols,
            rows,
        } => match run_complete(CompleteCommand {
            prefix,
            cursor_row,
            cursor_col,
            cols,
            rows,
            daemon_mode: daemon,
            shift_tab_sequence: shift_tab_hex
                .as_deref()
                .and_then(protocol::decode_hex_bytes),
            stale_bytes: stale_hex
                .as_deref()
                .and_then(protocol::decode_hex_bytes)
                .unwrap_or_default(),
            reuse_token,
            context_key,
            prev_popup_row,
            prev_popup_height,
        }) {
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
            let cfg = config::Config::load();
            let auto_insert_unambiguous = cfg.auto_insert_unambiguous;
            let theme = cfg.theme();
            match run_render(
                prefix,
                cursor_row,
                cursor_col,
                selected,
                &theme,
                auto_insert_unambiguous,
            ) {
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
