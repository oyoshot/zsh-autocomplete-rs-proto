use zsh_autocomplete_rs::app::App;
use zsh_autocomplete_rs::candidate::Candidate;
use zsh_autocomplete_rs::cli::{Cli, Command, DaemonAction};
use zsh_autocomplete_rs::handoff::compute_reuse_token;
use zsh_autocomplete_rs::{client, config, daemon, input, tty, ui};

use clap::Parser;
use std::io::{self, BufRead, Read, Write};
use std::process;
use tty::TtyGuard;

enum AppResult {
    Selected(String, String),
    DismissedWithSpace(String),
    Cancelled(Option<String>),
}

fn run_complete(
    prefix: String,
    cursor_row: u16,
    cursor_col: u16,
    bindings: &config::KeyBindings,
    theme: &config::Theme,
) -> io::Result<i32> {
    let candidates: Vec<Candidate> = io::stdin()
        .lock()
        .lines()
        .map_while(Result::ok)
        .filter(|line| !line.is_empty())
        .map(|line| Candidate::parse_line(&line))
        .collect();

    if candidates.is_empty() {
        return Ok(1);
    }

    let mut app = App::new(candidates, prefix, cursor_row, cursor_col);
    if app.filtered_indices.is_empty() {
        return Ok(1);
    }
    let mut guard = TtyGuard::new()?;

    // Scroll terminal to ensure blank space below cursor for popup
    ui::render::ensure_space(&mut guard.tty, &mut app)?;
    app.select_first();
    ui::render::draw(&mut guard.tty, &app, theme)?;

    let result = loop {
        match input::read_action(bindings)? {
            input::Action::MoveDown => {
                app.move_down();
                ui::render::draw(&mut guard.tty, &app, theme)?;
            }
            input::Action::MoveUp => {
                app.move_up();
                ui::render::draw(&mut guard.tty, &app, theme)?;
            }
            input::Action::PageDown => {
                app.page_down();
                ui::render::draw(&mut guard.tty, &app, theme)?;
            }
            input::Action::PageUp => {
                app.page_up();
                ui::render::draw(&mut guard.tty, &app, theme)?;
            }
            input::Action::Resize(cols, rows) => {
                ui::render::clear(&mut guard.tty, &app)?;
                app.set_term_size(cols, rows);
                ui::render::ensure_space(&mut guard.tty, &mut app)?;
                ui::render::draw(&mut guard.tty, &app, theme)?;
            }
            input::Action::Confirm => {
                ui::render::clear(&mut guard.tty, &app)?;
                break match app.selected_candidate() {
                    Some(c) => AppResult::Selected(c.text.clone(), c.kind.clone()),
                    None => AppResult::Cancelled(Some(app.filter_text.clone())),
                };
            }
            input::Action::DismissWithSpace => {
                ui::render::clear(&mut guard.tty, &app)?;
                break AppResult::DismissedWithSpace(format!("{} ", app.filter_text));
            }
            input::Action::Cancel => {
                ui::render::clear(&mut guard.tty, &app)?;
                let text = if app.filter_text != app.prefix {
                    Some(app.filter_text.clone())
                } else {
                    None
                };
                break AppResult::Cancelled(text);
            }
            input::Action::TypeChar(c) => {
                ui::render::clear(&mut guard.tty, &app)?;
                app.type_char(c);
                if app.filtered_indices.is_empty() {
                    break AppResult::Cancelled(Some(app.filter_text.clone()));
                }
                app.select_first();
                ui::render::draw(&mut guard.tty, &app, theme)?;
            }
            input::Action::Backspace => {
                ui::render::clear(&mut guard.tty, &app)?;
                if !app.backspace() {
                    break AppResult::Cancelled(None);
                }
                if app.filtered_indices.is_empty() || app.filter_text.len() < app.prefix.len() {
                    break AppResult::Cancelled(Some(app.filter_text.clone()));
                }
                app.select_first();
                ui::render::draw(&mut guard.tty, &app, theme)?;
            }
            input::Action::None => {}
        }
    };

    drop(guard);

    match result {
        AppResult::Selected(text, kind) => {
            let c = Candidate {
                text,
                description: String::new(),
                kind,
            };
            print!("{}", c.text_with_suffix());
            Ok(0)
        }
        AppResult::DismissedWithSpace(text) => {
            print!("{}", text);
            Ok(2)
        }
        AppResult::Cancelled(Some(text)) => {
            print!("{}", text);
            Ok(1)
        }
        AppResult::Cancelled(None) => Ok(1),
    }
}

fn run_render(
    prefix: String,
    cursor_row: u16,
    cursor_col: u16,
    theme: &config::Theme,
) -> io::Result<i32> {
    // Read raw stdin before trying daemon (we need it for both paths)
    let raw_stdin: Vec<u8> = {
        let mut buf = Vec::new();
        io::stdin().lock().read_to_end(&mut buf)?;
        buf
    };

    // Try daemon first
    if let Ok(resp) = client::try_daemon_render(&prefix, cursor_row, cursor_col, &raw_stdin) {
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
    ui::render::draw_popup_only(&mut tty, &app, theme)?;

    let popup = ui::popup::Popup::compute(&app);
    let candidates_tsv = std::str::from_utf8(&raw_stdin).unwrap_or("");
    let reuse_token = compute_reuse_token(&app.prefix, candidates_tsv, &app, &popup);
    println!(
        "popup_row={} popup_height={} cursor_row={} reuse_token={}",
        popup.row, popup.height, app.cursor_row, reuse_token
    );

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
    let cfg = config::Config::load();
    let bindings = cfg.key_bindings();
    let theme = cfg.theme();
    match cli.command {
        Command::Complete {
            prefix,
            cursor_row,
            cursor_col,
        } => match run_complete(prefix, cursor_row, cursor_col, &bindings, &theme) {
            Ok(code) => process::exit(code),
            Err(e) => {
                let _ = crossterm::terminal::disable_raw_mode();
                eprintln!("error: {}", e);
                process::exit(1);
            }
        },
        Command::Render {
            prefix,
            cursor_row,
            cursor_col,
        } => match run_render(prefix, cursor_row, cursor_col, &theme) {
            Ok(code) => process::exit(code),
            Err(e) => {
                eprintln!("error: {}", e);
                process::exit(1);
            }
        },
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
