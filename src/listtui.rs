use crate::api;
use crate::command::{Cmd, CmdType};
use crate::filterlist::FilterList;
use crate::git::GitUser;
use crate::input;
use crossterm::event::{Event, KeyCode, KeyEventKind};
use ratatui::{
    layout::{Constraint, Layout, Margin, Position},
    style::{Style, Stylize},
    text::{Line, Span, Text},
    widgets::{
        Block, List, ListDirection, ListItem, ListState, Padding, Paragraph, Scrollbar,
        ScrollbarOrientation, ScrollbarState,
    },
    Frame,
};
use Constraint::{Length, Min};

pub enum GiermError {
    NotFoundError,
}

enum ListTuiMode {
    Full,
    Inline,
}

enum InputMode {
    Repo,
    Username,
}

struct ListSearchTui {
    user: crate::git::User,
    git_user: Option<crate::git::GitUser>,
    searched_username: String,
    list: FilterList,
    command: crate::command::CmdType,
    mode: ListTuiMode,
    input_mode: InputMode,
    cursor: crate::cursor::Cursor,
    // Cursor pos
}

impl ListSearchTui {
    fn new(
        user: crate::git::User,
        git_user: Option<crate::git::GitUser>,
        searched_username: String,
        command: crate::command::CmdType,
        list: FilterList,
    ) -> Self {
        Self {
            user,
            git_user,
            searched_username,
            command,
            list,
            mode: ListTuiMode::Full,
            input_mode: InputMode::Repo,
            cursor: crate::cursor::Cursor::new(),
        }
    }

    async fn run(&mut self) -> Option<Cmd> {
        // TODO: inline mode
        let mut terminal = ratatui::init();
        let cmd: Option<Cmd>;
        loop {
            terminal
                .draw(|frame| self.draw(frame))
                .expect("failed to draw frame");
            match self.handle_events().await {
                Ok((true, command_opt)) => {
                    cmd = command_opt;
                    break;
                }
                Ok((false, command)) => {}
                _ => {}
            }
        }
        ratatui::restore();
        return cmd;
    }

    fn get_command(&mut self) -> Option<Cmd> {
        match self.command {
            crate::command::CmdType::CLONE => {
                if let Some(repo_i) = self.list.state.get_selected_index() {
                    let filtered_list = self.list.get_filtered();
                    let repo_name = filtered_list
                        .get(repo_i)
                        .expect("Index should have an item");
                    let (user_name, ssh) = match &self.git_user {
                        Some(u) => (u.username.clone(), false),
                        None => (self.user.git.username.clone(), true),
                    };
                    let url = crate::git::get_clone_url(&user_name, &repo_name, ssh);
                    let mut cmd = Cmd::new_git_cmd(CmdType::CLONE);
                    cmd.push_arg(url);
                    return Some(cmd);
                }
                return None;
            }
            _ => return None,
        }
    }

    fn changed_username(&self) -> bool {
        if let Some(u) = &self.git_user {
            return self.searched_username.to_lowercase().trim()
                != u.username.to_lowercase().trim();
        } else {
            return true;
        }
    }

    fn update_selected_user(&mut self, new_gituser: Option<GitUser>) {
        match new_gituser {
            Some(gu) => {
                let all_repos: Vec<String> = gu.repos.keys().cloned().collect();
                self.list.set_list(all_repos);
                self.git_user = Some(gu);
            }
            _ => {
                let all_repos: Vec<String> = self.user.git.repos.keys().cloned().collect();
                self.searched_username.clear();
                self.list.set_list(all_repos);
                self.git_user = None;
            }
        }
    }

    async fn fetch_new_gituser(&mut self) {
        if self.searched_username.trim().is_empty() {
            self.update_selected_user(None);
            self.input_mode = InputMode::Repo;
            return;
        }
        let git_u = api::search_gituser(&self.user, &self.searched_username).await;
        match git_u {
            Some(gu) => {
                self.update_selected_user(Some(gu));
                self.input_mode = InputMode::Repo;
            }
            _ => {}
        }
    }

    async fn change_input_mode(&mut self) {
        match self.input_mode {
            InputMode::Repo => {
                self.input_mode = InputMode::Username;
                self.cursor.reset();
            }
            InputMode::Username => {
                if self.changed_username() {
                    self.fetch_new_gituser().await;
                } else {
                    self.input_mode = InputMode::Repo;
                    self.cursor.reset();
                }
            }
        }
    }

    fn handle_input(&mut self, c: char) {
        match self.input_mode {
            InputMode::Repo => {
                self.cursor.insert_at_cursor(&mut self.list.filter, c);
            }
            InputMode::Username => {
                self.cursor.insert_at_cursor(&mut self.searched_username, c);
            }
        };
    }

    fn handle_backspace(&mut self) {
        match self.input_mode {
            InputMode::Repo => {
                self.cursor.remove_at_cursor(&mut self.list.filter);
            }
            InputMode::Username => {
                self.cursor.remove_at_cursor(&mut self.searched_username);
            }
        }
    }

    async fn handle_events(&mut self) -> std::io::Result<(bool, Option<Cmd>)> {
        match crossterm::event::read()? {
            Event::Key(key) if key.kind == KeyEventKind::Press => match key.code {
                KeyCode::Esc => return Ok((true, None)),
                KeyCode::Up => match self.input_mode {
                    InputMode::Repo => self.list.state.next(),
                    _ => {}
                },
                KeyCode::Down => match self.input_mode {
                    InputMode::Repo => self.list.state.previous(),
                    _ => {}
                },
                KeyCode::Left => match self.input_mode {
                    InputMode::Repo => {
                        self.cursor.c_left(self.list.filter.len());
                    }
                    InputMode::Username => {
                        self.cursor.c_left(self.searched_username.len());
                    }
                },

                KeyCode::Right => {
                    self.cursor.c_right();
                }
                KeyCode::Enter => match self.input_mode {
                    InputMode::Username => self.fetch_new_gituser().await,
                    InputMode::Repo => {
                        let cmd = self.get_command();
                        return Ok((true, cmd));
                    }
                },
                KeyCode::Tab => self.change_input_mode().await,
                KeyCode::Backspace => self.handle_backspace(),
                KeyCode::Char(c) => self.handle_input(c),
                _ => {}
            },
            _ => {}
        }
        Ok((false, None))
    }

    fn draw_list(&mut self, frame: &mut Frame) {
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

        frame.set_cursor_position(Position::new(
            filter_area.x + self.list.filter.len() as u16 + 2 - self.cursor.offset as u16,
            filter_area.y,
        ));

        frame.render_widget(p_matches, matches_area);
        frame.render_widget(p_filter, filter_area);
    }

    fn draw_search_user(&mut self, frame: &mut Frame) {
        let vertical = Layout::vertical([Min(0), Length(1), Length(1)]);
        let [_, info_area, input_area] = vertical.areas(frame.area());

        let info_text = Paragraph::new(Text::from(Line::from(vec![Span::styled(
            "Input git username:",
            Style::new().blue(),
        )])));

        let input_line = Paragraph::new(Text::from(Line::from(vec![
            Span::styled("=> ", Style::new().blue()),
            Span::styled(self.searched_username.clone(), Style::default()),
        ])));

        frame.set_cursor_position(Position::new(
            input_area.x + self.searched_username.len() as u16 + 3 - self.cursor.offset as u16,
            input_area.y,
        ));

        frame.render_widget(info_text, info_area);
        frame.render_widget(input_line, input_area);
    }

    fn draw(&mut self, frame: &mut Frame) {
        match self.input_mode {
            InputMode::Repo => self.draw_list(frame),
            InputMode::Username => self.draw_search_user(frame),
        }
    }
}

pub async fn run_list_selector(
    user: crate::git::User,
    username: String,
    filter: String,
    command: crate::command::CmdType,
) -> Result<(), GiermError> {
    let mut list_tui: ListSearchTui;
    if let Some(git_user) = crate::api::search_gituser(&user, &username).await {
        let all_repos: Vec<String> = git_user.repos.keys().cloned().collect();
        let fl = FilterList::new(all_repos, filter);
        list_tui = ListSearchTui::new(user, Some(git_user), username, command, fl);
    } else {
        let all_repos: Vec<String> = user.git.repos.keys().cloned().collect();
        let fl = FilterList::new(all_repos, filter);
        let not_found = !username.is_empty() && username != user.git.username;
        list_tui = ListSearchTui::new(user, None, "".to_string(), command, fl);
        if not_found {
            list_tui.input_mode = InputMode::Username;
        }
    }

    let cmd = list_tui.run().await;
    if let Some(mut command) = cmd {
        let cmd_str = command.to_string();
        let input_res = input::ask_path("Enter file path:".to_string(), &cmd_str);
        match input_res {
            Ok((true, input)) => {
                command.push_arg(input.trim().to_string());
                match command.exec(false) {
                    Ok(_) => {}
                    Err(_e) => {}
                }
            }
            _ => {}
        }
    }
    return Ok(());
}
