use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind};
use ratatui::{
    layout::{Constraint, Layout, Margin},
    style::{Color, Modifier, Style, Stylize},
    text::{Line, Span, Text},
    widgets::{
        Block, BorderType, Borders, List, ListDirection, ListItem, ListState, Paragraph, Scrollbar,
        ScrollbarOrientation, ScrollbarState,
    },
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
pub struct StateL {
    state: ListState,
    items_len: usize,
}

impl StateL {
    pub fn new(len: usize) -> Self {
        Self {
            state: ListState::default(),
            items_len: len,
        }
    }

    pub fn next(&mut self) {
        let i = self
            .state
            .selected()
            .map_or(0, |i| (i + 1) % self.items_len);
        self.state.select(Some(i));
    }

    pub fn previous(&mut self) {
        let i = self
            .state
            .selected()
            .map_or(0, |i| (i + self.items_len - 1) % self.items_len);
        self.state.select(Some(i));
    }

    pub fn get_selected_index(&self) -> Option<usize> {
        return self.state.selected();
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
    repo_list_state: StateL,
    repo_list: Vec<String>,
    commit_list: StateL, // Can be users or searched users commits
    search_user: String,
    search_repo: String,
    status_text: String,
    blocks_on_left: u8,
    blocks_on_right: u8,
    last_block: Option<u8>,
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
        let repos_state = StateL::new((&user).git.repos.keys().len());
        let mut repos: Vec<String> = user.git.repos.keys().cloned().collect();
        repos.sort_by(|x, y| {
            user.git
                .repos
                .get(y)
                .unwrap()
                .updated_at
                .cmp(&user.git.repos.get(x).unwrap().updated_at)
        });
        Self {
            user,
            selected_block: 0,
            repo_list_state: repos_state,
            repo_list: repos,
            commit_list: StateL::new(0),
            search_user,
            search_repo,
            status_text,
            blocks_on_left,
            blocks_on_right,
            last_block: None,
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

    fn next_block(&mut self) {
        if self.selected_block >= self.blocks_on_left {
            self.selected_block += 1;
            if self.selected_block == self.blocks_on_left + self.blocks_on_right {
                self.selected_block = self.blocks_on_left;
            }
        } else {
            self.selected_block = (self.selected_block + 1) % self.blocks_on_left;
        }
    }

    fn previous_block(&mut self) {
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

    fn selected_repo_name(&self) -> Option<String> {
        let repo_index = self.repo_list_state.get_selected_index()?;
        return Some(self.repo_list[repo_index].clone());
    }

    fn goto_right(&mut self) {
        self.last_block = Some(self.selected_block);
        self.selected_block = self.blocks_on_left;
    }

    async fn handle_events(&mut self) -> std::io::Result<bool> {
        match event::read()? {
            Event::Key(key) if key.kind == KeyEventKind::Press => match key.code {
                KeyCode::Char('q') => return Ok(true),
                KeyCode::Up => match block_type(self.selected_block) {
                    BlockType::Repos => self.repo_list_state.previous(),
                    BlockType::Commits => self.commit_list.previous(),
                    _ => {}
                },
                KeyCode::Down => match block_type(self.selected_block) {
                    BlockType::Repos => self.repo_list_state.next(),
                    BlockType::Commits => self.commit_list.next(),
                    _ => {}
                },
                KeyCode::Left => {
                    self.previous_block();
                }
                KeyCode::Right => {
                    self.next_block();
                }
                KeyCode::Enter => match block_type(self.selected_block) {
                    BlockType::Profile => {}
                    BlockType::Repos => {
                        if self.repo_list_state.state != ListState::default() {
                            let repo_name = self.selected_repo_name().expect("Expected repo index");
                            let repo = self.user.git.repos.get(&repo_name).unwrap();
                            if repo.commits.is_empty() {
                                let commits: Vec<crate::git::Commit> =
                                    crate::api::fetch_repo_commits(&self.user, &repo).await;
                                if let Some(repo) = self.user.git.repos.get_mut(&repo.name.clone())
                                {
                                    repo.commits = commits;
                                    self.commit_list.items_len = repo.commits.len();
                                }
                                self.status_text =
                                    format!("Fetched {} commits", self.commit_list.items_len);
                            } else {
                                self.commit_list.items_len = repo.commits.len();
                            }
                        }
                        self.commit_list.state = ListState::default();
                        self.goto_right();
                    }
                    BlockType::Search => {}
                    BlockType::Info => {}
                    BlockType::Commits => {}
                    BlockType::SearchResults => {}
                    _ => {}
                },
                KeyCode::Esc => {
                    self.status_text = "".to_string();
                    if let Some(b) = self.last_block {
                        self.selected_block = b;
                        self.last_block = None;
                    }
                }
                _ => {}
            },
            // handle other events
            _ => {}
        }
        Ok(false)
    }

    fn draw(&mut self, frame: &mut Frame) {
        let block_highlight_style = Style::new().green();
        let status_area_height = if self.status_text.is_empty() { 2 } else { 3 };

        let vertical = Layout::vertical([Min(0)]);
        let [main_area] = vertical.areas(frame.area());
        let horizontal = Layout::horizontal([Fill(1), Fill(2)]);
        let [left_area, right_area] = horizontal.areas(main_area);

        let left_vertical =
            Layout::vertical([Length(6), Min(0), Length(8), Length(status_area_height)]);
        let [profile_area, repo_list_area, search_area, status_area] =
            left_vertical.areas(left_area);

        let right_vertical = Layout::vertical([Length(10), Min(10), Min(10)]);
        let [info_area, commit_list_area, search_result_area] = right_vertical.areas(right_area);

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

        let repo_list_block = List::new(self.repo_list.clone())
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
            .highlight_symbol("")
            .repeat_highlight_symbol(true)
            .direction(ListDirection::TopToBottom);

        frame.render_stateful_widget(
            &repo_list_block,
            repo_list_area,
            &mut self.repo_list_state.state,
        );

        let scrollbar = Scrollbar::new(ScrollbarOrientation::VerticalRight)
            .begin_symbol(Some("↑"))
            .end_symbol(Some("↓"));
        let scrollbar_margin = Margin {
            vertical: 1,
            horizontal: 0,
        };

        let mut repo_list_scrollbar_state = ScrollbarState::new(self.repo_list_state.items_len)
            .position(self.repo_list_state.state.selected().unwrap_or(0));
        frame.render_stateful_widget(
            scrollbar.clone(),
            repo_list_area.inner(scrollbar_margin),
            &mut repo_list_scrollbar_state,
        );

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

        let repo_name = self.selected_repo_name();
        let commit_items: Vec<String>;
        let mut info_lines = vec![];
        match repo_name {
            Some(name) => {
                // TODO: selected repo can also be a searched user's repo
                let repo = self.user.git.repos.get(&name).unwrap();
                // TODO: different printing
                commit_items = repo.commits.iter().map(|c| c.to_string()).collect();
                info_lines.push(Line::from(vec![Span::styled(
                    repo.name.clone(),
                    Style::default(),
                )]));
                info_lines.push(Line::from(vec![
                    Span::styled("Description: ", Style::default()),
                    Span::styled(repo.description.clone(), Style::default()),
                ]));
                info_lines.push(Line::from(vec![
                    Span::styled("Language: ", Style::default()),
                    Span::styled(repo.language.clone(), Style::default()),
                ]));
                info_lines.push(Line::from(vec![
                    Span::styled("Last updated: ", Style::default()),
                    Span::styled(repo.updated_at.clone().to_string(), Style::default()),
                ]));
                info_lines.push(Line::from(vec![
                    Span::styled("Commits: ", Style::default()),
                    Span::styled(repo.commits.len().to_string(), Style::default()),
                ]));
            }
            None => {
                commit_items = Vec::new();
            }
        };

        let text = Text::from(info_lines);
        let info_block = Paragraph::new(text).block(
            Block::bordered()
                .title("Info")
                .border_type(BorderType::Rounded)
                .border_style(if block_type(self.selected_block) == BlockType::Info {
                    block_highlight_style
                } else {
                    Style::default()
                }),
        );
        frame.render_widget(info_block, info_area);

        let commit_list_block = List::new(commit_items)
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
            .highlight_symbol("")
            .repeat_highlight_symbol(true)
            .direction(ListDirection::TopToBottom);
        frame.render_stateful_widget(
            &commit_list_block,
            commit_list_area,
            &mut self.commit_list.state,
        );

        let mut commit_list_scrollbar_state = ScrollbarState::new(self.commit_list.items_len)
            .position(self.commit_list.state.selected().unwrap_or(0));
        frame.render_stateful_widget(
            scrollbar,
            commit_list_area.inner(scrollbar_margin),
            &mut commit_list_scrollbar_state,
        );

        let search_result_block = Block::bordered()
            .title("Results")
            .border_type(BorderType::Rounded)
            .border_style(
                if block_type(self.selected_block) == BlockType::SearchResults {
                    block_highlight_style
                } else {
                    Style::default()
                },
            );
        frame.render_widget(search_result_block, search_result_area);
    }
}
