use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind};
use ratatui::{
    layout::{Constraint, Layout},
    style::{Color, Modifier, Style, Stylize},
    text::{Line, Span, Text},
    widgets::{Block, BorderType, Borders, List, ListDirection, ListItem, ListState, Paragraph},
    Frame,
};
use Constraint::{Fill, Length, Min};

pub fn run_tui(user: crate::git::User) {
    let mut tui = Tui::new(
        user,
        "Username".to_string(),
        "Repo name".to_string(),
        "Status text".to_string(),
        3,
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

#[derive(PartialEq)]
enum BlockType {
    Profile,
    Repos,
    Search,
    Default,
}

fn block_type(b_i: u8) -> BlockType {
    match b_i {
        0 => BlockType::Profile,
        1 => BlockType::Repos,
        2 => BlockType::Search,
        _ => BlockType::Default,
    }
}

struct Tui {
    user: crate::git::User,
    selected_block: u8,
    repo_list: StatefulList<String>,
    search_user: String,
    search_repo: String,
    status_text: String,
    blocks_n: u8,
}

impl Tui {
    pub fn new(
        user: crate::git::User,
        search_user: String,
        search_repo: String,
        status_text: String,
        blocks: u8,
    ) -> Self {
        let repos =
            StatefulList::with_items((&user).git.repos.keys().cloned().collect::<Vec<String>>());
        Self {
            user,
            selected_block: 0,
            repo_list: repos,
            search_user,
            search_repo,
            status_text,
            blocks_n: blocks,
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

    pub fn next_block(&mut self) {
        self.selected_block = (self.selected_block + 1) % self.blocks_n;
    }

    pub fn previous_block(&mut self) {
        self.selected_block = (self.selected_block + self.blocks_n - 1) % self.blocks_n;
    }

    fn handle_events(&mut self) -> std::io::Result<bool> {
        match event::read()? {
            Event::Key(key) if key.kind == KeyEventKind::Press => match key.code {
                KeyCode::Char('q') => return Ok(true),
                KeyCode::Up => {
                    if self.selected_block == 1 {
                        self.repo_list.previous();
                    }
                }
                KeyCode::Down => {
                    if self.selected_block == 1 {
                        self.repo_list.next();
                    }
                }
                KeyCode::Left => {
                    self.previous_block();
                }
                KeyCode::Right => {
                    self.next_block();
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
            Layout::vertical([Length(6), Min(0), Length(8), Length(status_area_height)]);
        let [profile_area, repos_area, search_area, status_area] = left_vertical.areas(left_area);

        // Selected text highlight
        // Paragraph::new(status_text).block(Block::default())
        //     .bg(ratatui::prelude::Color::LightBlue)

        let profile_block = Block::bordered()
            .title(self.user.git.username.clone())
            .border_type(BorderType::Rounded)
            .border_style(if block_type(self.selected_block) == BlockType::Profile {
                Style::new().blue()
            } else {
                Style::default()
            });
        frame.render_widget(&profile_block, profile_area);

        let mut lines = vec![];
        lines.push(Line::from(vec![
            Span::styled("Name: ", Style::default()),
            Span::styled(self.user.git.name.clone(), Style::default()),
        ]));
        lines.push(Line::from(vec![
            Span::styled("Email: ", Style::default()),
            Span::styled(self.user.git.email.clone(), Style::default()),
        ]));
        lines.push(Line::from(vec![
            Span::styled("Bio: ", Style::default()),
            Span::styled(self.user.git.bio.clone(), Style::default()),
        ]));
        lines.push(Line::from(vec![
            Span::styled("Rate remaining: ", Style::default()),
            Span::styled(self.user.rate().to_string(), Style::default()),
        ]));
        let text = Text::from(lines);
        let p = Paragraph::new(text);
        frame.render_widget(p, profile_block.inner(profile_area));

        let repos_list = List::new(self.repo_list.items.clone())
            .block(
                Block::bordered()
                    .title("Repos")
                    .border_type(BorderType::Rounded)
                    .border_style(if block_type(self.selected_block) == BlockType::Repos {
                        Style::new().blue()
                    } else {
                        Style::default()
                    }),
            )
            .style(Style::new().white())
            .highlight_style(Style::new().italic().blue())
            .highlight_symbol(">>")
            .repeat_highlight_symbol(true)
            .direction(ListDirection::TopToBottom);

        frame.render_stateful_widget(&repos_list, repos_area, &mut self.repo_list.state);

        let search_block = Block::bordered()
            .title("Search")
            .border_type(BorderType::Rounded)
            .border_style(if block_type(self.selected_block) == BlockType::Search {
                Style::new().blue()
            } else {
                Style::default()
            });
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
