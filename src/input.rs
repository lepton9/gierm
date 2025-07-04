use crate::autocomplete::AutoComplete;
use crossterm::cursor;
use crossterm::event::{Event, KeyCode, KeyEventKind};
use crossterm::{
    cursor::{MoveLeft, MoveRight, RestorePosition, SavePosition},
    ExecutableCommand,
};
use std::{io::Write, process::Command};

pub fn ask_input(prompt: String, input_beg: &String) -> std::io::Result<(bool, String)> {
    crossterm::terminal::enable_raw_mode()?;
    let mut cursor = crate::cursor::Cursor::new();
    let mut input: String = String::default();
    let mut cout = std::io::stdout();
    writeln!(&mut cout, "{}", prompt)?;
    cout.execute(cursor::MoveToColumn(0))?;
    write!(&mut cout, "{}[2K > {} {}", 27 as char, input_beg, input)?;
    cout.flush()?;
    loop {
        match crossterm::event::read()? {
            Event::Key(key) if key.kind == KeyEventKind::Press => match key.code {
                KeyCode::Esc => {
                    crossterm::terminal::disable_raw_mode()?;
                    println!();
                    return Ok((false, "".to_string()));
                }
                KeyCode::Left => {
                    if cursor.c_left(input.len()) {
                        cout.execute(MoveLeft(1))?;
                    }
                }
                KeyCode::Right => {
                    if cursor.c_right() {
                        cout.execute(MoveRight(1))?;
                    }
                }
                KeyCode::Enter => {
                    crossterm::terminal::disable_raw_mode()?;
                    println!();
                    return Ok((true, input));
                }
                KeyCode::Backspace => {
                    if cursor.remove_at_cursor(&mut input) {
                        cout.execute(MoveLeft(1))?;
                    }
                }
                KeyCode::Char(c) => {
                    cursor.insert_at_cursor(&mut input, c);
                    cout.execute(MoveRight(1))?;
                }
                _ => {}
            },
            _ => {}
        }
        cout.execute(SavePosition)?;
        cout.execute(cursor::MoveToColumn(0))?;
        write!(&mut cout, "{}[2K > {} {}", 27 as char, input_beg, input)?;
        cout.flush()?;
        cout.execute(RestorePosition)?;
    }
}

pub fn ask_path(prompt: String, input_beg: &String) -> std::io::Result<(bool, String)> {
    crossterm::terminal::enable_raw_mode()?;
    let mut cursor = crate::cursor::Cursor::new();
    let mut input: String = String::default();
    let mut complete = AutoComplete::new();
    let mut choosing_match: bool = false;
    let mut cout = std::io::stdout();
    writeln!(&mut cout, "{}", prompt)?;
    cout.execute(cursor::MoveToColumn(0))?;
    write!(&mut cout, "{}[2K > {} {}", 27 as char, input_beg, input)?;
    cout.flush()?;
    loop {
        match crossterm::event::read()? {
            Event::Key(key) if key.kind == KeyEventKind::Press => match key.code {
                KeyCode::Esc => {
                    crossterm::terminal::disable_raw_mode()?;
                    println!();
                    return Ok((false, "".to_string()));
                }
                KeyCode::Left => {
                    if cursor.c_left(input.len()) {
                        cout.execute(MoveLeft(1))?;
                    }
                }
                KeyCode::Right => {
                    if cursor.c_right() {
                        cout.execute(MoveRight(1))?;
                    }
                }
                KeyCode::Enter => {
                    if choosing_match {
                    } else {
                        crossterm::terminal::disable_raw_mode()?;
                        println!();
                        return Ok((true, input));
                    }
                }
                KeyCode::Backspace => {
                    if cursor.remove_at_cursor(&mut input) {
                        complete.update_input(input.clone());
                        cout.execute(MoveLeft(1))?;
                    }
                }
                KeyCode::Tab => {
                    match complete.complete() {
                        Some(true) => {
                            choosing_match = false;
                            input = complete.get_input();
                        }
                        Some(false) => {
                            //
                            choosing_match = true;
                        }
                        _ => {
                            //
                            choosing_match = false;
                        }
                    }
                }
                KeyCode::Char(c) => {
                    cursor.insert_at_cursor(&mut input, c);
                    complete.update_input(input.clone());
                    cout.execute(MoveRight(1))?;
                }
                _ => {}
            },
            _ => {}
        }
        cout.execute(SavePosition)?;
        cout.execute(cursor::MoveToColumn(0))?;
        cout.execute(crossterm::terminal::Clear(
            crossterm::terminal::ClearType::CurrentLine,
        ))?;
        if choosing_match {
            if let Some(m) = complete.selected() {
                write!(
                    &mut cout,
                    " > {} {}",
                    input_beg,
                    complete.input_with_match(m)
                )?;
            } else {
                write!(&mut cout, " > {} {}", input_beg, input)?;
            }
            // TODO: display the matches
        } else {
            write!(&mut cout, " > {} {}", input_beg, input)?;
        }
        cout.flush()?;
        cout.execute(RestorePosition)?;
    }
}

pub fn display_items(items: Vec<String>) {}
