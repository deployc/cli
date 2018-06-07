use std::fmt::Display;
use std::io::{stdin, stdout, Stdout, Write};

use colored::*;
use prettytable::row::Row;
use prettytable::{format, Table};
use slug::slugify;
use termion;
use termion::event::Key;
use termion::input::TermRead;
use termion::raw::{IntoRawMode, RawTerminal};

use commands::CommandError;

const POINTER: &'static str = "❯";

pub fn prompt<S>(s: S) -> Option<String>
where
    S: Display,
{
    let stdin = stdin();

    print!("{}", s);
    stdout().flush().unwrap();

    let mut s = String::new();
    match stdin.read_line(&mut s) {
        Ok(u) if u > 0 => Some(s),
        _ => None,
    }
}

pub fn prompt_password<S>(s: S) -> Option<String>
where
    S: Display,
{
    let mut stdin = stdin();
    let mut stdout = stdout();

    print!("{}", s);
    stdout.flush().unwrap();
    if let Ok(Some(p)) = stdin.read_passwd(&mut stdout) {
        println!("");
        Some(p)
    } else {
        None
    }
}

pub fn prompt_credentials() -> Result<(String, String), CommandError> {
    let username =
        prompt("  Username: ".bold()).ok_or(CommandError::with_message("Invalid username."))?;

    let password = prompt_password("  Password: ".bold())
        .ok_or(CommandError::with_message("Invalid password."))?
        .trim()
        .to_string();

    Ok((slugify(username), password))
}

pub fn print_table(header: Row, rows: Vec<Row>) {
    let mut table = Table::new();
    table.set_titles(header);
    for row in rows {
        table.add_row(row);
    }

    table.set_format(*format::consts::FORMAT_CLEAN);
    table.printstd();
}

fn write_options<T>(
    stdout: &mut RawTerminal<Stdout>,
    options: &Vec<T>,
    choice: usize,
    curr_idx: Option<usize>,
) where
    T: Display,
{
    for (i, opt) in options.iter().enumerate() {
        if choice == i {
            write!(
                stdout,
                " {} {}\r\n",
                POINTER.cyan().bold(),
                format!("{}", opt).cyan().bold()
            ).unwrap();
        } else if curr_idx.map_or(false, |c| c == i) {
            let opt = format!("{}", opt).dimmed().bold();
            write!(stdout, "   {} {}\r\n", opt, "(current)".dimmed()).unwrap();
        } else {
            write!(stdout, "   {}\r\n", opt).unwrap();
        }
    }

    stdout.flush().unwrap();
}

pub fn get_option<T>(options: &Vec<T>, current_value: Option<T>) -> Result<Option<&T>, CommandError>
where
    T: Display,
    T: PartialEq,
{
    let stdin = stdin();
    let mut stdout = stdout().into_raw_mode().unwrap();
    let num_options = options.len() as u16;

    write!(
        stdout,
        "{}{}",
        termion::clear::CurrentLine,
        termion::cursor::Hide
    ).unwrap();

    let curr_idx = current_value.map_or(None, |opt| options.iter().position(|o| o == &opt));
    let mut choice = if curr_idx.is_some() && curr_idx.unwrap() == 0 {
        1
    } else {
        0
    };
    let mut canceled = false;

    write_options(&mut stdout, &options, choice, curr_idx);
    for c in stdin.keys() {
        write!(
            stdout,
            "{}{}",
            termion::clear::CurrentLine,
            termion::cursor::Up(num_options)
        ).unwrap();
        stdout.flush().unwrap();
        match c.unwrap() {
            Key::Esc | Key::Ctrl('c') => {
                canceled = true;
                break;
            }
            Key::Up if choice > 0 => {
                if curr_idx.map_or(false, |i| i == choice - 1 && i != 0) {
                    choice -= 2;
                } else if curr_idx.map_or(true, |i| i != choice - 1) {
                    choice -= 1;
                }
            }
            Key::Down if choice < options.len() - 1 => {
                if curr_idx.map_or(false, |i| i == choice + 1 && i != options.len() - 1) {
                    choice += 2;
                } else if curr_idx.map_or(true, |i| i != choice + 1) {
                    choice += 1;
                }
            }
            Key::Char('\n') => {
                write!(
                    stdout,
                    "{}{}",
                    termion::cursor::Show,
                    termion::clear::CurrentLine
                ).unwrap();
                return Ok(Some(&options[choice]));
            }
            _ => {}
        }

        write_options(&mut stdout, &options, choice, curr_idx);
    }

    write!(stdout, "{}", termion::cursor::Show).unwrap();

    if canceled {
        return Ok(None);
    }

    Err(CommandError::with_message("Could not choose option."))
}

pub fn ellipsis(s: &String, max_len: usize) -> String {
    if s.len() > max_len {
        format!("{}…", s.chars().take(max_len).collect::<String>())
    } else {
        s.clone()
    }
}
