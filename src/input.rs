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
    cout.execute(crossterm::terminal::Clear(
        crossterm::terminal::ClearType::FromCursorDown,
    ))?;
    write!(&mut cout, " > {} {}", input_beg, input)?;
    cout.flush()?;
    loop {
        display_path_content(&mut cout, choosing_match, &complete, input_beg, &input)?;
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
                        match complete.accept_selected_match() {
                            Ok(_) => {
                                choosing_match = false;
                                input = complete.get_input();
                            }
                            _ => {}
                        }
                    } else {
                        crossterm::terminal::disable_raw_mode()?;
                        println!();
                        return Ok((true, input));
                    }
                }
                KeyCode::Backspace => {
                    choosing_match = false;
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
                    if choosing_match {
                        match complete.accept_selected_match() {
                            Ok(_) => {
                                choosing_match = false;
                                input = complete.get_input();
                                cursor.reset();
                            }
                            _ => {}
                        }
                    }
                    cursor.insert_at_cursor(&mut input, c);
                    complete.update_input(input.clone());
                    // TODO: cursor placement
                    cout.execute(MoveRight(1))?;
                }
                _ => {}
            },
            _ => {}
        }
    }
}

pub fn calc_scroll_amount(items: &Vec<String>, item_width: usize, spacing: usize) -> usize {
    let (term_width, term_height) = {
        let (w, h) = crossterm::terminal::size().unwrap_or((80, 24));
        (w as usize, h as usize)
    };
    let items_on_line = if (item_width + spacing) > 0 {
        term_width / (item_width + spacing)
    } else {
        1
    };
    let needed_height: usize = (items.len() + items_on_line - 1) / items_on_line;
    let line = cursor::position().unwrap_or((0, term_height as u16 - 1)).1 as usize + 1;
    return if needed_height > (term_height - line) {
        needed_height - (term_height - line)
    } else {
        0
    };
}

pub fn display_path_content(
    cout: &mut std::io::Stdout,
    choosing_match: bool,
    complete: &AutoComplete,
    input_beg: &String,
    input: &String,
) -> std::io::Result<()> {
    let match_item_width = 30;
    let item_spacing = 2;
    cout.execute(SavePosition)?;
    cout.execute(cursor::MoveToColumn(0))?;
    cout.execute(crossterm::terminal::Clear(
        crossterm::terminal::ClearType::CurrentLine,
    ))?;
    if choosing_match {
        if let Some(m) = complete.selected() {
            write!(cout, " > {} {}", input_beg, complete.input_with_match(m))?;
        } else {
            write!(cout, " > {} {}", input_beg, input)?;
        }

        cout.execute(RestorePosition)?;
        let matches = complete.get_matches();
        let scroll = calc_scroll_amount(&matches, match_item_width, item_spacing);
        if scroll > 0 {
            cout.execute(crossterm::terminal::ScrollUp(scroll as u16))?;
            cout.execute(cursor::MoveUp(scroll as u16))?;
        }
        cout.execute(SavePosition)?;
        display_items(
            cout,
            matches,
            complete.selected_index(),
            match_item_width,
            item_spacing,
        )?;
        cout.execute(RestorePosition)?;
    } else {
        cout.execute(crossterm::terminal::Clear(
            crossterm::terminal::ClearType::FromCursorDown,
        ))?;
        write!(cout, " > {} {}", input_beg, input)?;
        cout.execute(RestorePosition)?;
    }
    cout.flush()?;
    return Ok(());
}

pub fn display_items(
    cout: &mut std::io::Stdout,
    items: Vec<String>,
    selected: Option<usize>,
    item_width: usize,
    spacing: usize,
) -> std::io::Result<()> {
    let (term_width, term_height) = {
        let (w, h) = crossterm::terminal::size()?;
        (w as usize, h as usize)
    };
    let mut current_line_length = 0;
    let total_item_width = item_width + spacing;

    cout.execute(cursor::MoveToColumn(0))?;
    cout.execute(cursor::MoveToNextLine(1))?;
    cout.execute(crossterm::terminal::Clear(
        crossterm::terminal::ClearType::FromCursorDown,
    ))?;
    for (index, item) in items.iter().enumerate() {
        if current_line_length + total_item_width > term_width {
            cout.execute(cursor::MoveToNextLine(1))?;
            cout.execute(cursor::MoveToColumn(0))?;
            cout.execute(crossterm::terminal::Clear(
                crossterm::terminal::ClearType::CurrentLine,
            ))?;
            current_line_length = 0;
        }

        if Some(index) == selected {
            cout.execute(crossterm::style::SetAttribute(
                crossterm::style::Attribute::Reverse,
            ))?;
            write!(cout, "{:<w$}", item, w = item_width)?;
            cout.execute(crossterm::style::SetAttribute(
                crossterm::style::Attribute::NoReverse,
            ))?;
            write!(cout, "{}", " ".repeat(spacing))?;
        } else {
            write!(cout, "{:<w$}{}", item, " ".repeat(spacing), w = item_width)?;
        }
        current_line_length += total_item_width;
    }
    return Ok(());
}
