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

pub enum GiermError {
    NotFoundError,
}

struct FilterList {
    state: StateL,
    list: Vec<String>,
    filter: String,
}

impl FilterList {
    fn new(list: Vec<String>, filter: String) -> Self {
        Self {
            state: StateL::new(list.len()),
            list,
            filter,
        }
    }

    fn get_filtered(&mut self) -> Vec<String> {
        let l: Vec<String> = self
            .list
            .clone()
            .into_iter()
            .filter(|rn| rn.to_lowercase().contains(&self.filter.to_lowercase()))
            .collect();
        self.state.new_size(l.len());
        return l;
    }

    fn set_filter(&mut self, filter: String) {
        self.filter = filter;
    }

    fn filter_append(&mut self, c: char) {
        self.filter.push(c);
    }

    fn filter_remove_last(&mut self) {
        self.filter.pop();
    }

    // fn set_filtered(&mut self, list: Vec<String>, filter: String) {
    //     self.list = list
    //         .into_iter()
    //         .filter(|rn| rn.to_lowercase().contains(&filter.to_lowercase()))
    //         .collect();
    //     self.filter = filter;
    // }
}

enum SearchUIMode {
    Full,
    Inline,
}

struct ListSearchTui {
    user: crate::git::User,
    git_user: Option<crate::git::GitUser>,
    list: FilterList,
    command: crate::args::Command,
    mode: SearchUIMode,
    // Cursor pos
}

impl ListSearchTui {
    fn new(
        user: crate::git::User,
        git_user: Option<crate::git::GitUser>,
        command: crate::args::Command,
        list: FilterList,
    ) -> Self {
        Self {
            user,
            git_user,
            command,
            list,
            mode: SearchUIMode::Full,
        }
    }

    async fn run(&mut self) {
        // TODO: inline mode
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

    async fn handle_events(&mut self) -> std::io::Result<bool> {
        match event::read()? {
            Event::Key(key) if key.kind == KeyEventKind::Press => match key.code {
                KeyCode::Esc => return Ok(true),
                KeyCode::Up => self.list.state.next(), // move list up
                KeyCode::Down => self.list.state.previous(), // move list down
                KeyCode::Left => {}                    // move filter left
                KeyCode::Right => {}                   // move filter right
                KeyCode::Enter => {}                   // select item
                KeyCode::Backspace => self.list.filter_remove_last(), // remove char from filter
                KeyCode::Tab => {}                     // switch filter mode to search diff user
                KeyCode::Char(c) => {
                    self.list.filter_append(c);
                }
                _ => {}
            },
            _ => {}
        }
        Ok(false)
    }

    fn draw(&mut self, frame: &mut Frame) {
        let vertical = Layout::vertical([Min(0), Length(1), Length(1)]);
        let [list_area, matches_area, filter_area] = vertical.areas(frame.area());

        let filtered_list = self.list.get_filtered();
        let mut list_items: Vec<ListItem> = Vec::new();

        let mut list_iter = filtered_list.iter();
        while let Some(item) = list_iter.next() {
            let filter_start = item.find(&self.list.filter).unwrap_or(0);
            let (beg, rest) = item.split_at(filter_start);
            let (mid, end) = rest.split_at(self.list.filter.len());
            let li = ListItem::new(Text::from(Line::from(vec![
                Span::styled(beg, Style::default()),
                Span::styled(mid, Style::new().green()),
                Span::styled(end, Style::default()),
            ])));
            list_items.push(li);
        }

        let list_block = List::new(list_items)
            .block(Block::new().padding(Padding::left(2)))
            .style(Style::new())
            .highlight_style(Style::new().italic().blue())
            .highlight_symbol("> ")
            .repeat_highlight_symbol(true)
            .direction(ListDirection::BottomToTop);
        // .direction(ListDirection::TopToBottom); // If inline mode, and change up and down

        let p_matches = Paragraph::new(Text::from(Line::from(vec![
            Span::styled(
                format!("  {}/{}", filtered_list.len(), self.list.list.len()),
                Style::new().light_red(),
            ),
            Span::styled("", Style::default().gray().dim()),
        ])));

        let p_filter = Paragraph::new(Text::from(Line::from(vec![
            Span::styled("> ", Style::new().blue()),
            Span::styled(self.list.filter.clone(), Style::default()),
        ])));

        let scrollbar = Scrollbar::new(ScrollbarOrientation::VerticalLeft)
            .thumb_style(Style::new().gray().dim())
            .track_symbol(None)
            .begin_symbol(None)
            .end_symbol(None);
        let scrollbar_margin = Margin {
            vertical: 1,
            horizontal: 0,
        };
        let mut list_scrollbar_state = ScrollbarState::new(filtered_list.len())
            .position(filtered_list.len() - self.list.state.state.selected().unwrap_or(0));

        if filtered_list.len() > 0 && self.list.state.state == ListState::default() {
            self.list.state.state.select(Some(0));
        }

        frame.render_stateful_widget(&list_block, list_area, &mut self.list.state.state);
        frame.render_stateful_widget(
            scrollbar,
            list_area.inner(scrollbar_margin),
            &mut list_scrollbar_state,
        );

        frame.render_widget(p_matches, matches_area);
        frame.render_widget(p_filter, filter_area);
    }
}

pub async fn run_list_selector(
    user: crate::git::User,
    username: String,
    filter: String,
    command: crate::args::Command,
) -> Result<(), GiermError> {
    if username.is_empty() || username == user.git.username {
        let all_repos: Vec<String> = user.git.repos.keys().cloned().collect();
        let fl = FilterList::new(all_repos, filter);
        let mut list_tui = ListSearchTui::new(user, None, command, fl);
        list_tui.run().await;
        return Ok(());
    } else if let Some(mut git_user) = crate::api::search_gituser(&user, &username).await {
        let all_repos: Vec<String> = git_user.repos.keys().cloned().collect();
        let fl = FilterList::new(all_repos, filter);
        let mut list_tui = ListSearchTui::new(user, Some(git_user), command, fl);
        list_tui.run().await;
        return Ok(());
    } else {
        return Err(GiermError::NotFoundError);
    }
}

pub async fn run_tui(user: crate::git::User) {
    let mut tui = Tui::new(
        user,
        "lepton9".to_string(),
        "".to_string(),
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

#[derive(PartialEq)]
enum BlockType {
    Profile,
    Repos,
    Search,
    Info,
    Commits,
    SearchResults,
    SearchUser,
    SearchRepo,
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
        6 => BlockType::SearchUser,
        7 => BlockType::SearchRepo,
        _ => BlockType::Default,
    }
}

fn block_type_to_u8(block_type: BlockType) -> u8 {
    return block_type as u8;
}

// TODO: use ListSelector
struct SearchedUser {
    user: crate::git::GitUser,
    repo_list_state: StateL,
    repo_list: Vec<String>,
    commit_list: StateL,
    filter: String,
}

// TODO: add func to modify filter and update the searched repos
// instead of fetching the user again
// Only if username field is the same
impl SearchedUser {
    pub fn new(user: crate::git::GitUser, filter: String) -> Self {
        let repos_state = StateL::new((&user).repos.keys().len());
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
            repo_list_state: repos_state,
            repo_list: repos,
            commit_list: StateL::new(0),
            filter,
        }
    }

    fn selected_repo_name(&self) -> Option<String> {
        let repo_index = self.repo_list_state.get_selected_index()?;
        return Some(self.repo_list[repo_index].clone());
    }
}

struct Tui {
    user: crate::git::User,
    selected_block: u8,
    repo_list_state: StateL,
    repo_list: Vec<String>,
    commit_list: StateL,
    search_user: String,
    search_repo: String,
    status_text: String,
    blocks_on_left: u8,
    blocks_on_right: u8,
    last_block: Option<u8>,
    searched_user: Option<SearchedUser>,
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

    // TODO: fix movement in search form fields
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

    fn goto_block(&mut self, block_type: BlockType) {
        self.last_block = Some(self.selected_block);
        self.selected_block = block_type_to_u8(block_type);
    }

    fn goto_right(&mut self) {
        self.goto_block(block_type(self.blocks_on_left));
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
                KeyCode::Enter => {
                    self.status_text = "".to_string();
                    match block_type(self.selected_block) {
                        BlockType::Profile => {}
                        BlockType::Repos => {
                            if self.repo_list_state.state == ListState::default() {
                                self.repo_list_state.next();
                            } else {
                                let repo_name =
                                    self.selected_repo_name().expect("Expected repo index");
                                let repo = self.user.git.repos.get(&repo_name).unwrap();
                                if repo.commits.is_empty() {
                                    let commits: Vec<crate::git::Commit> =
                                        crate::api::fetch_repo_commits(&self.user, &repo).await;
                                    if let Some(repo) =
                                        self.user.git.repos.get_mut(&repo.name.clone())
                                    {
                                        repo.commits = commits;
                                        self.commit_list.items_len = repo.commits.len();
                                    }
                                    self.status_text =
                                        format!("Fetched {} commits", self.commit_list.items_len);
                                } else {
                                    self.commit_list.items_len = repo.commits.len();
                                }
                                self.commit_list.state = ListState::default();
                                self.goto_right();
                            }
                        }
                        BlockType::Search => {
                            // TODO: goto SearchUser instead to the right
                            // from SearchUser or SearchRepo goto right on enter
                            // self.goto_block(BlockType::SearchUser);

                            if self.searched_user.is_none()
                                || self.search_user.to_lowercase()
                                    != self
                                        .searched_user
                                        .as_ref()
                                        .unwrap()
                                        .user
                                        .name
                                        .to_lowercase()
                            {
                                let search_result =
                                    crate::api::search_gituser(&self.user, &self.search_user).await;
                                self.searched_user = match search_result {
                                    Some(mut user) => {
                                        user.repos =
                                            crate::api::fetch_repos(&self.user, &self.search_user)
                                                .await;

                                        let found: SearchedUser =
                                            SearchedUser::new(user, self.search_repo.clone());
                                        self.status_text = format!(
                                            "Found {} with {} repos",
                                            self.search_user,
                                            found.user.repos.len()
                                        );
                                        Some(found)
                                    }
                                    None => {
                                        self.status_text =
                                            format!("No user named {} found", self.search_user);
                                        None
                                    }
                                };
                            }
                            self.goto_right();
                        }
                        BlockType::Info => {}
                        BlockType::Commits => {
                            if self.repo_list_state.state != ListState::default() {
                                if let Some(index) = self.commit_list.get_selected_index() {
                                    let username = &self.user.git.username;
                                    let repo_name = self.selected_repo_name().unwrap();
                                    let repo = self.user.git.repos.get(&repo_name.clone()).unwrap();
                                    let commit =
                                        repo.commits.get(index).map(|commit| commit).unwrap();
                                    if !commit.info.is_some() {
                                        let commit_info = crate::api::fetch_commit_info(
                                            &self.user,
                                            username.clone(),
                                            repo_name.clone(),
                                            commit.sha.clone(),
                                        )
                                        .await;
                                        {
                                            if let Some(commit) = self
                                                .user
                                                .git
                                                .repos
                                                .get_mut(&repo_name)
                                                .and_then(|repo| repo.commits.get_mut(index))
                                            {
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
                        BlockType::SearchResults => {}
                        _ => {}
                    }
                }
                KeyCode::Esc => {
                    self.status_text = "".to_string();
                    if let Some(b) = self.last_block {
                        self.selected_block = b;
                        self.last_block = None;
                    }
                }
                _ => {}
            },
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
            .border_style(
                if block_type(self.selected_block) == BlockType::SearchUser {
                    block_highlight_style
                } else {
                    Style::default()
                },
            );
        let repo_search_block = Block::bordered()
            .border_type(BorderType::Rounded)
            .title("Repo")
            .border_style(
                if block_type(self.selected_block) == BlockType::SearchRepo {
                    block_highlight_style
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
