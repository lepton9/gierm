use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind};
use ratatui::{
    layout::{Constraint, Layout},
    style::{Color, Modifier, Style, Stylize},
    text::{Line, Span},
    widgets::Block,
    widgets::Paragraph,
    Frame,
};
use Constraint::{Fill, Length, Min};

pub fn run_tui() {
    let mut terminal = ratatui::init();
    loop {
        terminal.draw(draw).expect("failed to draw frame");
        if handle_events().unwrap() {
            break;
        }
    }
    ratatui::restore();
}

fn handle_events() -> std::io::Result<bool> {
    match event::read()? {
        Event::Key(key) if key.kind == KeyEventKind::Press => match key.code {
            KeyCode::Char('q') => return Ok(true),
            KeyCode::Up => {
                //
            }
            KeyCode::Down => {
                //
            }
            KeyCode::Left => {
                //
            }
            KeyCode::Right => {
                //
            }
            KeyCode::Enter => {
                //
            }
            KeyCode::Esc => {
                //
            }
            _ => {}
        },
        // handle other events
        _ => {}
    }
    Ok(false)
}

struct TuiLayout {
    blocks: Vec<TuiBlock>,
}

enum BlockType {
    PROFILE,
    STATUS,
    SEARCH,
}

struct TuiBlock {
    selected: bool,
    height: u16,
    block_type: BlockType,
}

pub fn draw(frame: &mut Frame) {
    let status_text = "Status text".to_string();
    let status_area_height = if status_text.is_empty() { 2 } else { 3 };
    let status_block = Block::bordered().title("Status");

    let vertical = Layout::vertical([Min(0)]);
    let [main_area] = vertical.areas(frame.area());
    let horizontal = Layout::horizontal([Fill(1), Fill(2)]);
    let [left_area, right_area] = horizontal.areas(main_area);

    let left_vertical =
        Layout::vertical([Length(3), Min(0), Length(8), Length(status_area_height)]);
    let [profile_area, commands_area, search_area, status_area] = left_vertical.areas(left_area);

    // Selected text highlight
    // Paragraph::new(status_text).block(Block::default())
    //     .bg(ratatui::prelude::Color::LightBlue)
    //
    // Selected block style
    // .border_style(Style::new().blue()),

    frame.render_widget(
        Block::bordered()
            .title("Profile")
            .border_style(Style::new().blue()),
        profile_area,
    );
    frame.render_widget(
        Block::bordered()
            .title("Commands")
            .border_style(Style::new()),
        commands_area,
    );

    let search_block = Block::bordered().title("Search").border_style(Style::new());
    let user_search_block = Block::bordered().title("User").border_style(Style::new());
    let repo_search_block = Block::bordered().title("Repo").border_style(Style::new());

    let [user_search_area, repo_search_area] =
        Layout::vertical([Constraint::Percentage(50), Constraint::Percentage(50)])
            .areas(search_block.inner(search_area));
    frame.render_widget(&search_block, search_area);
    frame.render_widget(&user_search_block, user_search_area);
    frame.render_widget(&repo_search_block, repo_search_area);
    frame.render_widget(
        Paragraph::new("Username").block(Block::default()),
        user_search_block.inner(user_search_area),
    );
    frame.render_widget(
        Paragraph::new("Repo name").block(Block::default()),
        repo_search_block.inner(repo_search_area),
    );

    frame.render_widget(&status_block, status_area);
    frame.render_widget(
        Paragraph::new(status_text).block(Block::default()),
        status_block.inner(status_area),
    );

    frame.render_widget(Block::bordered().title("Right"), right_area);
}
