use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind};
use ratatui::{
    layout::{Constraint, Layout},
    style::{Color, Modifier, Style, Stylize},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, List, ListDirection, ListItem, ListState, Paragraph},
    Frame,
};
use Constraint::{Fill, Length, Min};

pub fn run_tui(user: crate::git::User) {
    let mut tui = Tui::new(
        user,
        0,
        "Username".to_string(),
        "Repo name".to_string(),
        "Status text".to_string(),
    );
    tui.run();
}

#[derive(Debug, Default)]
pub struct StatefulList<T> {
    state: ListState,
    items: Vec<T>,
}

impl<T> StatefulList<T> {
    pub fn with_items(items: Vec<T>) -> Self {
        Self {
            state: ListState::default(),
            items,
        }
    }
    pub fn next(&mut self) {
        let i = match self.state.selected() {
            Some(i) => {
                if i >= self.items.len() - 1 {
                    0
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        self.state.select(Some(i));
    }
    pub fn previous(&mut self) {
        let i = match self.state.selected() {
            Some(i) => {
                if i == 0 {
                    self.items.len() - 1
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.state.select(Some(i));
    }

    pub fn get_selected(&self) -> Option<&T> {
        self.state.selected().map(|index| &self.items[index])
    }
}

struct Tui {
    user: crate::git::User,
    selected_block: u16,
    repo_list: StatefulList<String>,
    search_user: String,
    search_repo: String,
    status_text: String,
}

impl Tui {
    pub fn new(
        user: crate::git::User,
        selected_block: u16,
        search_user: String,
        search_repo: String,
        status_text: String,
    ) -> Self {
        let repos =
            StatefulList::with_items((&user).git.repos.keys().cloned().collect::<Vec<String>>());
        Self {
            user,
            selected_block,
            repo_list: repos,
            search_user,
            search_repo,
            status_text,
        }
    }

    fn run(&mut self) {
        let mut terminal = ratatui::init();
        loop {
            terminal
                .draw(|frame| self.draw(frame))
                .expect("failed to draw frame");
            if self.handle_events().unwrap() {
                break;
            }
        }
        ratatui::restore();
    }

    fn handle_events(&mut self) -> std::io::Result<bool> {
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

    pub fn draw(&mut self, frame: &mut Frame) {
        let status_area_height = if self.status_text.is_empty() { 2 } else { 3 };

        let vertical = Layout::vertical([Min(0)]);
        let [main_area] = vertical.areas(frame.area());
        let horizontal = Layout::horizontal([Fill(1), Fill(2)]);
        let [left_area, right_area] = horizontal.areas(main_area);

        let left_vertical =
            Layout::vertical([Length(3), Min(0), Length(8), Length(status_area_height)]);
        let [profile_area, repos_area, search_area, status_area] = left_vertical.areas(left_area);

        // Selected text highlight
        // Paragraph::new(status_text).block(Block::default())
        //     .bg(ratatui::prelude::Color::LightBlue)
        //
        // Selected block style
        // .border_style(Style::new().blue()),

        frame.render_widget(
            Block::bordered()
                .title(self.user.git.username.clone())
                .border_type(BorderType::Rounded)
                .border_style(Style::new().blue()),
            profile_area,
        );

        let repos_list = List::new(self.repo_list.items.clone())
            .block(
                Block::bordered()
                    .title("Repos")
                    .border_type(BorderType::Rounded),
            )
            .style(Style::new().white())
            .highlight_style(Style::new().italic())
            .highlight_symbol(">>")
            .repeat_highlight_symbol(true)
            .direction(ListDirection::TopToBottom);

        // self.repo_list.next();
        frame.render_stateful_widget(&repos_list, repos_area, &mut self.repo_list.state);

        let search_block = Block::bordered()
            .title("Search")
            .border_type(BorderType::Rounded)
            .border_style(Style::new());
        let user_search_block = Block::bordered()
            .border_type(BorderType::Rounded)
            .title("User")
            .border_style(Style::new());
        let repo_search_block = Block::bordered()
            .border_type(BorderType::Rounded)
            .title("Repo")
            .border_style(Style::new());

        let [user_search_area, repo_search_area] =
            Layout::vertical([Constraint::Percentage(50), Constraint::Percentage(50)])
                .areas(search_block.inner(search_area));
        frame.render_widget(&search_block, search_area);
        frame.render_widget(&user_search_block, user_search_area);
        frame.render_widget(&repo_search_block, repo_search_area);
        frame.render_widget(
            Paragraph::new(self.search_user.clone()).block(Block::default()),
            user_search_block.inner(user_search_area),
        );
        frame.render_widget(
            Paragraph::new(self.search_repo.clone()).block(Block::default()),
            repo_search_block.inner(repo_search_area),
        );

        let status_block = Block::bordered()
            .title("Status")
            .border_type(BorderType::Rounded);
        frame.render_widget(&status_block, status_area);
        frame.render_widget(
            Paragraph::new(self.status_text.clone()).block(Block::default()),
            status_block.inner(status_area),
        );

        frame.render_widget(
            Block::bordered()
                .border_type(BorderType::Rounded)
                .title("Right"),
            right_area,
        );
    }
}
