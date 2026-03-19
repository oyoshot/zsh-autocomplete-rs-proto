mod app;
mod candidate;
mod cli;
mod config;
mod fuzzy;
mod input;
mod tty;
mod ui;

use app::App;
use candidate::Candidate;
use clap::Parser;
use cli::{Cli, Command};
use std::io::{self, BufRead};
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
) -> io::Result<i32> {
    let candidates: Vec<Candidate> = io::stdin()
        .lock()
        .lines()
        .filter_map(|line| line.ok())
        .filter(|line| !line.is_empty())
        .map(|line| Candidate::parse_line(&line))
        .collect();

    if candidates.is_empty() {
        return Ok(1);
    }

    let mut app = App::new(candidates, prefix, cursor_row, cursor_col);
    let mut guard = TtyGuard::new()?;

    // Scroll terminal to ensure blank space below cursor for popup
    ui::render::ensure_space(&mut guard.tty, &mut app)?;
    ui::render::draw(&mut guard.tty, &app)?;

    let result = loop {
        match input::read_action(bindings)? {
            input::Action::MoveDown => {
                app.move_down();
                ui::render::draw(&mut guard.tty, &app)?;
            }
            input::Action::MoveUp => {
                app.move_up();
                ui::render::draw(&mut guard.tty, &app)?;
            }
            input::Action::PageDown => {
                app.page_down();
                ui::render::draw(&mut guard.tty, &app)?;
            }
            input::Action::PageUp => {
                app.page_up();
                ui::render::draw(&mut guard.tty, &app)?;
            }
            input::Action::Confirm => {
                ui::render::clear(&mut guard.tty, &app)?;
                break match app.selected_candidate() {
                    Some(c) => AppResult::Selected(c.text.clone(), c.kind.clone()),
                    None => AppResult::Cancelled(None),
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
                if app.filtered.is_empty() {
                    break AppResult::Cancelled(Some(app.filter_text.clone()));
                }
                ui::render::draw(&mut guard.tty, &app)?;
            }
            input::Action::Backspace => {
                ui::render::clear(&mut guard.tty, &app)?;
                app.backspace();
                if app.filtered.is_empty() || app.filter_text.len() < app.prefix.len() {
                    break AppResult::Cancelled(Some(app.filter_text.clone()));
                }
                ui::render::draw(&mut guard.tty, &app)?;
            }
            input::Action::None => {}
        }
    };

    drop(guard);

    match result {
        AppResult::Selected(text, kind) => {
            let suffix = match kind.as_str() {
                "directory" => {
                    if text.ends_with('/') {
                        ""
                    } else {
                        "/"
                    }
                }
                "command" | "alias" | "builtin" | "function" | "file" => " ",
                _ => "",
            };
            print!("{}{}", text, suffix);
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

fn main() {
    let cli = Cli::parse();
    let cfg = config::Config::load();
    let bindings = cfg.key_bindings();
    match cli.command {
        Command::Complete {
            prefix,
            cursor_row,
            cursor_col,
        } => match run_complete(prefix, cursor_row, cursor_col, &bindings) {
            Ok(code) => process::exit(code),
            Err(e) => {
                let _ = crossterm::terminal::disable_raw_mode();
                eprintln!("error: {}", e);
                process::exit(1);
            }
        },
    }
}
