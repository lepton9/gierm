use crate::layout::*;
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind};
use ratatui::{
    layout::{Constraint, Layout, Margin},
    style::{Color, Modifier, Style, Stylize},
    text::{Line, Span, Text},
    widgets::{
        Block, BorderType, Borders, List, ListDirection, ListItem, ListState, Padding, Paragraph,
        Scrollbar, ScrollbarOrientation, ScrollbarState,
    },
    Frame,
};
use Constraint::{Fill, Length, Min};

pub async fn run_tui(user: crate::git::User) {
    let mut tui = Tui::new(
        user,
        "lepton9".to_string(),
        "".to_string(),
        "Status text".to_string(),
    );
    tui.run().await;
}

// TODO : better layout?
fn create_layout(layout: &mut TuiLayout) {
    layout.add_col();
    layout.add_block(BlockType::Profile, 0);
    layout.add_block(BlockType::Repos, 0);
    let sub_layout = layout.add_layout(BlockType::Search, 0);
    sub_layout.add_col();
    sub_layout.add_block(BlockType::SearchUser, 0);
    sub_layout.add_block(BlockType::SearchRepo, 0);
    layout.add_col();
    layout.add_block(BlockType::Info, 1);
    layout.add_block(BlockType::Commits, 1);
    layout.add_block(BlockType::SearchResults, 1);
}

enum Mode {
    Tui,
    Input,
}

#[derive(Debug, Default)]
pub struct StateL {
    pub state: ListState,
    pub items_len: usize,
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

    pub fn new_size(&mut self, n: usize) {
        self.items_len = n;
        if self.items_len < self.get_selected_index().unwrap_or(0) {
            self.state.select(Some(self.items_len));
        }
    }

    pub fn get_selected_index(&self) -> Option<usize> {
        return self.state.selected();
    }
}

struct SearchedUser {
    pub user: crate::git::GitUser,
    repo_list: crate::filterlist::FilterList,
    commit_list: StateL,
}

impl SearchedUser {
    pub fn new(user: crate::git::GitUser, filter: String) -> Self {
        let mut repos: Vec<String> = user
            .repos
            .keys()
            .cloned()
            .filter(|rn| rn.to_lowercase().contains(&filter.to_lowercase()))
            .collect();
        repos.sort_by(|x, y| {
            user.repos
                .get(y)
                .unwrap()
                .updated_at
                .cmp(&user.repos.get(x).unwrap().updated_at)
        });
        Self {
            user,
            repo_list: crate::filterlist::FilterList::new(repos, filter),
            commit_list: StateL::new(0),
        }
    }

    fn selected_repo_name(&mut self) -> Option<String> {
        let repo_index = self.repo_list.get_index()?;
        return Some(self.repo_list.get_filtered()[repo_index].clone());
    }
}

struct Tui {
    mode: Mode,
    user: crate::git::User,
    layout: TuiLayout,
    repo_list_state: StateL,
    repo_list: Vec<String>,
    commit_list: StateL,
    search_user: String,
    search_repo: String,
    status_text: String,
    searched_user: Option<SearchedUser>,
    show_user_data: bool,
}

impl Tui {
    pub fn new(
        user: crate::git::User,
        search_user: String,
        search_repo: String,
        status_text: String,
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
        let mut lo = TuiLayout::new();
        create_layout(&mut lo);
        Self {
            mode: Mode::Tui,
            user,
            layout: lo,
            repo_list_state: repos_state,
            repo_list: repos,
            commit_list: StateL::new(0),
            search_user,
            search_repo,
            status_text,
            searched_user: None,
            show_user_data: true,
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

    fn selected_repo_name(&self) -> Option<String> {
        let repo_index = self.repo_list_state.get_selected_index()?;
        return Some(self.repo_list[repo_index].clone());
    }

    fn selected_repo_name_su(&mut self) -> Option<String> {
        let su = self.searched_user.as_mut()?;
        let repo_index = su.repo_list.get_index()?;
        let list = su.repo_list.get_filtered();
        return Some(list.get(repo_index).cloned()?);
    }

    fn repo_list_prev(&mut self) {
        if self.show_su_data() {
            self.searched_user
                .as_mut()
                .unwrap()
                .repo_list
                .state
                .previous();
        } else {
            self.repo_list_state.previous();
        }
    }

    fn repo_list_next(&mut self) {
        if self.show_su_data() {
            self.searched_user.as_mut().unwrap().repo_list.state.next();
        } else {
            self.repo_list_state.next();
        }
    }

    fn show_su_data(&self) -> bool {
        return self.searched_user.is_some() && !self.show_user_data;
    }

    fn set_status(&mut self, status: String) {
        self.status_text = status;
    }

    async fn handle_events(&mut self) -> std::io::Result<bool> {
        match self.mode {
            Mode::Tui => match event::read()? {
                Event::Key(key) if key.kind == KeyEventKind::Press => {
                    return self.handle_keys_tui(key.code).await;
                }
                _ => {}
            },
            Mode::Input => match event::read()? {
                Event::Key(key) if key.kind == KeyEventKind::Press => {
                    self.handle_keys_input(key.code).await;
                }
                _ => {}
            },
        }
        Ok(false)
    }

    async fn handle_keys_tui(&mut self, key_code: KeyCode) -> std::io::Result<bool> {
        match key_code {
            KeyCode::Char('q') => return Ok(true),
            KeyCode::Up | KeyCode::Char('k') => match self.layout.active_block().block_type() {
                BlockType::Repos => self.repo_list_prev(),
                BlockType::Commits => self.commit_list.previous(),
                _ => {}
            },
            KeyCode::Down | KeyCode::Char('j') => match self.layout.active_block().block_type() {
                BlockType::Repos => self.repo_list_next(),
                BlockType::Commits => self.commit_list.next(),
                _ => {}
            },
            KeyCode::Left | KeyCode::Char('h') => {
                self.layout.prev_block();
            }
            KeyCode::Right | KeyCode::Char('l') => {
                self.layout.next_block();
            }
            KeyCode::Enter | KeyCode::Tab => {
                self.set_status("".to_string());
                self.handle_enter().await;
            }
            KeyCode::Esc => {
                self.set_status("".to_string());
                if !self.layout.unselect_layout() {
                    if self.layout.active_block_pos().col == 0 {
                        self.show_user_data = true;
                    } else {
                        self.layout.prev_col();
                    }
                }
            }
            _ => {}
        }
        return Ok(false);
    }

    async fn handle_keys_input(&mut self, key_code: KeyCode) {
        match key_code {
            KeyCode::Backspace => match self.layout.active_block().block_type() {
                BlockType::SearchUser => {
                    self.search_user.pop();
                }
                BlockType::SearchRepo => {
                    self.search_repo.pop();
                }
                _ => {}
            },
            KeyCode::Char(c) => match self.layout.active_block().block_type() {
                BlockType::SearchUser => {
                    self.search_user.push(c);
                }
                BlockType::SearchRepo => {
                    self.search_repo.push(c);
                }
                _ => {}
            },
            KeyCode::Left => {
                self.layout.prev_block();
            }
            KeyCode::Right => {
                self.layout.next_block();
            }
            KeyCode::Enter | KeyCode::Tab => {
                self.handle_enter().await;
            }
            KeyCode::Esc => {
                self.mode = Mode::Tui;
            }
            _ => {}
        }
    }

    async fn search(&mut self) {
        if self.searched_user.is_none()
            || self.search_user.to_lowercase()
                != self
                    .searched_user
                    .as_ref()
                    .unwrap()
                    .user
                    .username
                    .to_lowercase()
        {
            let search_result = crate::api::search_gituser(&self.user, &self.search_user).await;
            self.searched_user = match search_result {
                Some(mut user) => {
                    user.repos = crate::api::fetch_repos(&self.user, &self.search_user).await;

                    let found: SearchedUser = SearchedUser::new(user, self.search_repo.clone());
                    self.set_status(format!(
                        "Found user {} with {} repos",
                        self.search_user,
                        found.user.repos.len()
                    ));
                    Some(found)
                }
                None => {
                    self.set_status(format!("No user found with '{}'", self.search_user));
                    None
                }
            };
        }
        if self.searched_user.is_some() {
            self.show_user_data = false;
            self.layout.unselect_layout();
            self.layout.next_col();
        }
    }

    async fn handle_enter(&mut self) {
        match self.layout.active_block().block_type() {
            BlockType::Profile => {}
            BlockType::Repos => {
                if self.repo_list_state.state == ListState::default() {
                    self.repo_list_state.next();
                } else {
                    let repo_name = self.selected_repo_name().expect("Expected repo index");
                    let repo = self.user.git.repos.get(&repo_name).unwrap();
                    if repo.commits.is_empty() {
                        let commits: Vec<crate::git::Commit> =
                            crate::api::fetch_repo_commits(&self.user, &repo).await;
                        if let Some(repo) = self.user.git.repos.get_mut(&repo.name.clone()) {
                            repo.commits = commits;
                            self.commit_list.items_len = repo.commits.len();
                        }
                        self.set_status(format!("Fetched {} commits", self.commit_list.items_len));
                    } else {
                        self.commit_list.items_len = repo.commits.len();
                    }
                    self.commit_list.state = ListState::default();
                    self.layout.next_col();
                }
            }
            BlockType::Search => {
                self.layout.select_layout();
            }
            BlockType::SearchUser | BlockType::SearchRepo => match self.mode {
                Mode::Input => {
                    self.search().await;
                    self.mode = Mode::Tui;
                }
                _ => self.mode = Mode::Input,
            },
            BlockType::Info => {}
            BlockType::Commits => {
                if self.repo_list_state.state != ListState::default() {
                    if let Some(index) = self.commit_list.get_selected_index() {
                        let username = &self.user.git.username;
                        let repo_name = self.selected_repo_name().unwrap();
                        let repo = self.user.git.repos.get(&repo_name.clone()).unwrap();
                        let commit = repo.commits.get(index).map(|commit| commit).unwrap();
                        if !commit.info.is_some() {
                            let commit_info = crate::api::fetch_commit_info(
                                &self.user,
                                username.clone(),
                                repo_name.clone(),
                                commit.sha.clone(),
                            )
                            .await;
                            {
                                if let Some(repo) = self.user.git.repos.get_mut(&repo_name) {
                                    if let Some(commit) = repo.commits.get_mut(index) {
                                        commit.info = Some(commit_info);
                                        self.status_text = format!(
                                            "Fetched commit info for {}",
                                            commit.sha_short()
                                        );
                                    }
                                }
                            }
                        }
                    }
                }
            }
            BlockType::SearchResults => {}
            _ => {}
        }
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

        let profile_block = Block::bordered()
            .title(self.user.git.username.clone())
            .border_type(BorderType::Rounded)
            .border_style(
                if self.layout.active_block().block_type() == BlockType::Profile {
                    block_highlight_style
                } else {
                    Style::default()
                },
            );
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

        let repo_list = match self.show_su_data() {
            true => self
                .searched_user
                .as_mut()
                .unwrap()
                .repo_list
                .get_filtered(),
            false => self.repo_list.clone(),
        };

        let mut repo_list_state = match self.show_su_data() {
            true => self
                .searched_user
                .as_mut()
                .unwrap()
                .repo_list
                .state
                .state
                .clone(),
            false => self.repo_list_state.state.clone(),
        };

        let mut repo_list_scrollbar_state = match self.show_su_data() {
            true => {
                let su = self.searched_user.as_ref().unwrap();
                ScrollbarState::new(su.repo_list.state.items_len)
                    .position(su.repo_list.get_index().unwrap_or(0))
            }
            false => ScrollbarState::new(self.repo_list_state.items_len)
                .position(self.repo_list_state.state.selected().unwrap_or(0)),
        };

        let repo_list_block = List::new(repo_list)
            .block(
                Block::bordered()
                    .title("Repos")
                    .border_type(BorderType::Rounded)
                    .border_style(
                        if self.layout.active_block().block_type() == BlockType::Repos {
                            block_highlight_style
                        } else {
                            Style::default()
                        },
                    ),
            )
            .style(Style::new().white())
            .highlight_style(Style::new().italic().blue())
            .highlight_symbol("")
            .repeat_highlight_symbol(true)
            .direction(ListDirection::TopToBottom);

        let scrollbar = Scrollbar::new(ScrollbarOrientation::VerticalRight)
            .begin_symbol(Some("↑"))
            .end_symbol(Some("↓"));
        let scrollbar_margin = Margin {
            vertical: 1,
            horizontal: 0,
        };

        frame.render_stateful_widget(&repo_list_block, repo_list_area, &mut repo_list_state);

        frame.render_stateful_widget(
            scrollbar.clone(),
            repo_list_area.inner(scrollbar_margin),
            &mut repo_list_scrollbar_state,
        );

        let search_block = Block::bordered()
            .title("Search")
            .border_type(BorderType::Rounded)
            .border_style(
                if self.layout.active_block().block_type() == BlockType::Search {
                    block_highlight_style
                } else {
                    Style::default()
                },
            );
        let user_search_block = Block::bordered()
            .border_type(BorderType::Rounded)
            .title("User")
            .border_style(
                if self.layout.active_block().block_type() == BlockType::SearchUser {
                    match self.mode {
                        Mode::Input => Style::new().blue(),
                        _ => block_highlight_style,
                    }
                } else {
                    Style::default()
                },
            );
        let repo_search_block = Block::bordered()
            .border_type(BorderType::Rounded)
            .title("Repo")
            .border_style(
                if self.layout.active_block().block_type() == BlockType::SearchRepo {
                    match self.mode {
                        Mode::Input => Style::new().blue(),
                        _ => block_highlight_style,
                    }
                } else {
                    Style::default()
                },
            );

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

        // TODO: show searched user repos and info
        if self.searched_user.is_some() {}

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
                .border_style(
                    if self.layout.active_block().block_type() == BlockType::Info {
                        block_highlight_style
                    } else {
                        Style::default()
                    },
                ),
        );
        frame.render_widget(info_block, info_area);

        let commit_list_block = List::new(commit_items)
            .block(
                Block::bordered()
                    .title("Commits")
                    .border_type(BorderType::Rounded)
                    .border_style(
                        if self.layout.active_block().block_type() == BlockType::Commits {
                            block_highlight_style
                        } else {
                            Style::default()
                        },
                    ),
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
                if self.layout.active_block().block_type() == BlockType::SearchResults {
                    block_highlight_style
                } else {
                    Style::default()
                },
            );
        frame.render_widget(search_result_block, search_result_area);
    }
}
