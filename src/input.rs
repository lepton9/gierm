use crossterm::cursor;
use crossterm::event::{Event, KeyCode, KeyEventKind};
use crossterm::{
    cursor::{MoveLeft, MoveRight, RestorePosition, SavePosition},
    ExecutableCommand,
};
use std::io::Write;

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
    let mut complete = crate::autocomplete::AutoComplete::new();
    let mut choosing_match: bool = false;
    let mut cout = std::io::stdout();
    writeln!(&mut cout, "{}", prompt)?;
    cout.execute(cursor::MoveToColumn(0))?;
    clear_below(&mut cout)?;
    write!(&mut cout, " > {} {}", input_beg, input)?;
    cout.flush()?;
    let ret = loop {
        display_path_content(&mut cout, choosing_match, &complete, input_beg, &input)?;
        if !choosing_match {
            update_cursor_pos(&mut cout, &cursor)?;
        }
        match crossterm::event::read()? {
            Event::Key(key) if key.kind == KeyEventKind::Press => match key.code {
                KeyCode::Esc => {
                    break Ok((false, "".to_string()));
                }
                KeyCode::Left => {
                    cursor.c_left(input.len());
                }
                KeyCode::Right => {
                    cursor.c_right();
                }
                KeyCode::Enter => {
                    let accepted = if choosing_match {
                        accept_match(&mut complete, &mut cursor, &mut choosing_match, &mut input)
                    } else {
                        false
                    };
                    if !accepted {
                        break Ok((true, input));
                    }
                }
                KeyCode::Backspace => {
                    let accepted = if choosing_match {
                        accept_match(&mut complete, &mut cursor, &mut choosing_match, &mut input)
                    } else {
                        false
                    };
                    if !accepted && cursor.remove_at_cursor(&mut input) {
                        complete.update_input(input.clone());
                    }
                }
                KeyCode::Tab => match complete.complete() {
                    Some(true) => {
                        choosing_match = false;
                        input = complete.get_input();
                        cursor.reset();
                    }
                    Some(false) => {
                        choosing_match = true;
                    }
                    _ => {
                        choosing_match = false;
                    }
                },
                KeyCode::BackTab => {
                    if choosing_match {
                        complete.select_previous();
                    }
                }
                KeyCode::Char(c) => {
                    if choosing_match {
                        accept_match(&mut complete, &mut cursor, &mut choosing_match, &mut input);
                    }
                    cursor.insert_at_cursor(&mut input, c);
                    complete.update_input(input.clone());
                }
                _ => {}
            },
            _ => {}
        }
    };
    crossterm::terminal::disable_raw_mode()?;
    println!();
    clear_below(&mut cout)?;
    return ret;
}

fn accept_match(
    complete: &mut crate::autocomplete::AutoComplete,
    cursor: &mut crate::cursor::Cursor,
    choosing_match: &mut bool,
    input: &mut String,
) -> bool {
    if let Ok(_) = complete.accept_selected_match() {
        *choosing_match = false;
        *input = complete.get_input();
        cursor.reset();
        return true;
    }
    return false;
}

pub fn clear_below(cout: &mut std::io::Stdout) -> std::io::Result<()> {
    cout.execute(crossterm::terminal::Clear(
        crossterm::terminal::ClearType::FromCursorDown,
    ))?;
    return Ok(());
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

pub fn update_cursor_pos(
    cout: &mut std::io::Stdout,
    cursor: &crate::cursor::Cursor,
) -> std::io::Result<()> {
    if cursor.offset > 0 {
        cout.execute(cursor::MoveLeft(cursor.offset as u16))?;
    }
    return Ok(());
}

pub fn display_path_content(
    cout: &mut std::io::Stdout,
    choosing_match: bool,
    complete: &crate::autocomplete::AutoComplete,
    input_beg: &String,
    input: &String,
) -> std::io::Result<()> {
    let match_item_width = 30;
    let item_spacing = 2;
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
        clear_below(cout)?;
        write!(cout, " > {} {}", input_beg, input)?;
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
    let (term_width, _term_height) = {
        let (w, h) = crossterm::terminal::size()?;
        (w as usize, h as usize)
    };
    let mut current_line_length = 0;
    let total_item_width = item_width + spacing;

    cout.execute(cursor::MoveToColumn(0))?;
    cout.execute(cursor::MoveToNextLine(1))?;
    clear_below(cout)?;
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
