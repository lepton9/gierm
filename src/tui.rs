use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind};
use ratatui::{
    layout::{Constraint, Layout},
    style::{Color, Modifier, Style, Stylize},
    text::{Line, Span, Text},
    widgets::{Block, BorderType, Borders, List, ListDirection, ListItem, ListState, Paragraph},
    Frame,
};
use Constraint::{Fill, Length, Min};

pub async fn run_tui(user: crate::git::User) {
    let mut tui = Tui::new(
        user,
        "Username".to_string(),
        "Repo name".to_string(),
        "Status text".to_string(),
        3,
        3,
    );
    tui.run().await;
}

#[derive(Debug, Default)]
pub struct StatefulList<T> {
    state: ListState,
    items: Vec<T>,
}

impl<T> StatefulList<T> {
    pub fn new(items: Vec<T>) -> Self {
        Self {
            state: ListState::default(),
            items,
        }
    }

    pub fn next(&mut self) {
        let i = self
            .state
            .selected()
            .map_or(0, |i| (i + 1) % self.items.len());
        self.state.select(Some(i));
    }

    pub fn previous(&mut self) {
        let i = self
            .state
            .selected()
            .map_or(0, |i| (i + self.items.len() - 1) % self.items.len());
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
    Info,
    Commits,
    SearchResults,
    Default,
}

fn block_type(b_i: u8) -> BlockType {
    match b_i {
        0 => BlockType::Profile,
        1 => BlockType::Repos,
        2 => BlockType::Search,
        3 => BlockType::Info,
        4 => BlockType::Commits,
        5 => BlockType::SearchResults,
        _ => BlockType::Default,
    }
}

struct Tui {
    user: crate::git::User,
    selected_block: u8,
    repo_list: StatefulList<String>, // TODO: keep only the state and use the Vec in User
    commit_list: StatefulList<String>,
    search_user: String,
    search_repo: String,
    status_text: String,
    blocks_on_left: u8,
    blocks_on_right: u8,
    searched_user: Option<crate::git::GitUser>,
}

impl Tui {
    pub fn new(
        user: crate::git::User,
        search_user: String,
        search_repo: String,
        status_text: String,
        blocks_on_left: u8,
        blocks_on_right: u8,
    ) -> Self {
        let repos = StatefulList::new((&user).git.repos.keys().cloned().collect::<Vec<String>>());
        Self {
            user,
            selected_block: 0,
            repo_list: repos,
            commit_list: StatefulList::new(Vec::new()),
            search_user,
            search_repo,
            status_text,
            blocks_on_left,
            blocks_on_right,
            searched_user: None,
        }
    }

    async fn run(&mut self) {
        let mut terminal = ratatui::init();
        loop {
            terminal
                .draw(|frame| self.draw(frame))
                .expect("failed to draw frame");
            if self.handle_events().await.unwrap() {
                break;
            }
        }
        ratatui::restore();
    }

    pub fn next_block(&mut self) {
        if self.selected_block >= self.blocks_on_left {
            self.selected_block += 1;
            if self.selected_block == self.blocks_on_left + self.blocks_on_right {
                self.selected_block = self.blocks_on_left;
            }
        } else {
            self.selected_block = (self.selected_block + 1) % self.blocks_on_left;
        }
    }

    pub fn previous_block(&mut self) {
        if self.selected_block >= self.blocks_on_left {
            self.selected_block -= 1;
            if self.selected_block < self.blocks_on_left {
                self.selected_block = self.blocks_on_left + self.blocks_on_right - 1;
            }
        } else {
            self.selected_block =
                (self.selected_block + self.blocks_on_left - 1) % self.blocks_on_left;
        }
    }

    async fn handle_events(&mut self) -> std::io::Result<bool> {
        match event::read()? {
            Event::Key(key) if key.kind == KeyEventKind::Press => match key.code {
                KeyCode::Char('q') => return Ok(true),
                KeyCode::Up => {
                    if block_type(self.selected_block) == BlockType::Repos {
                        self.repo_list.previous();
                    }
                }
                KeyCode::Down => {
                    if block_type(self.selected_block) == BlockType::Repos {
                        self.repo_list.next();
                    }
                }
                KeyCode::Left => {
                    self.previous_block();
                }
                KeyCode::Right => {
                    self.next_block();
                }
                KeyCode::Enter => match block_type(self.selected_block) {
                    BlockType::Profile => {}
                    BlockType::Repos => {
                        if self.repo_list.state != ListState::default() {
                            let repo_name = self
                                .repo_list
                                .get_selected()
                                .expect("No selected repo name");
                            let repo = self.user.git.repos.get(repo_name).unwrap();
                            if repo.commits.is_empty() {
                                let commits: Vec<crate::git::Commit> =
                                    crate::api::fetch_repo_commits(&self.user, &repo).await;
                                if let Some(repo) = self.user.git.repos.get_mut(repo_name) {
                                    repo.commits = commits;
                                    self.commit_list.items =
                                        repo.commits.iter().map(|c| c.to_string()).collect();
                                }
                            } else {
                                self.commit_list.items =
                                    repo.commits.iter().map(|c| c.to_string()).collect();
                            }
                        }
                    }
                    BlockType::Search => {}
                    BlockType::Info => {}
                    BlockType::Commits => {}
                    BlockType::SearchResults => {}
                    _ => {}
                },
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
        let block_highlight_style = Style::new().green();
        let status_area_height = if self.status_text.is_empty() { 2 } else { 3 };

        let vertical = Layout::vertical([Min(0)]);
        let [main_area] = vertical.areas(frame.area());
        let horizontal = Layout::horizontal([Fill(1), Fill(2)]);
        let [left_area, right_area] = horizontal.areas(main_area);

        let left_vertical =
            Layout::vertical([Length(6), Min(0), Length(8), Length(status_area_height)]);
        let [profile_area, repos_area, search_area, status_area] = left_vertical.areas(left_area);

        let right_vertical = Layout::vertical([Length(10), Min(10), Min(10)]);
        let [repo_info_area, commit_list_area, search_result_area] =
            right_vertical.areas(right_area);

        // Selected text highlight
        // Paragraph::new(status_text).block(Block::default())
        //     .bg(ratatui::prelude::Color::LightBlue)

        let profile_block = Block::bordered()
            .title(self.user.git.username.clone())
            .border_type(BorderType::Rounded)
            .border_style(if block_type(self.selected_block) == BlockType::Profile {
                block_highlight_style
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

        let repos_list_block = List::new(self.repo_list.items.clone())
            .block(
                Block::bordered()
                    .title("Repos")
                    .border_type(BorderType::Rounded)
                    .border_style(if block_type(self.selected_block) == BlockType::Repos {
                        block_highlight_style
                    } else {
                        Style::default()
                    }),
            )
            .style(Style::new().white())
            .highlight_style(Style::new().italic().blue())
            .highlight_symbol(">>")
            .repeat_highlight_symbol(true)
            .direction(ListDirection::TopToBottom);

        frame.render_stateful_widget(&repos_list_block, repos_area, &mut self.repo_list.state);

        let search_block = Block::bordered()
            .title("Search")
            .border_type(BorderType::Rounded)
            .border_style(if block_type(self.selected_block) == BlockType::Search {
                block_highlight_style
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
                .title("Info"),
            repo_info_area,
        );

        let commit_list_block = List::new(self.commit_list.items.clone())
            .block(
                Block::bordered()
                    .title("Commits")
                    .border_type(BorderType::Rounded)
                    .border_style(if block_type(self.selected_block) == BlockType::Commits {
                        block_highlight_style
                    } else {
                        Style::default()
                    }),
            )
            .style(Style::new().white())
            .highlight_style(Style::new().italic().blue())
            .highlight_symbol(">>")
            .repeat_highlight_symbol(true)
            .direction(ListDirection::TopToBottom);

        frame.render_stateful_widget(
            &commit_list_block,
            commit_list_area,
            &mut self.commit_list.state,
        );

        frame.render_widget(
            Block::bordered()
                .border_type(BorderType::Rounded)
                .title("Results"),
            search_result_area,
        );
    }
}
