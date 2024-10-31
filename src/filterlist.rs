use crate::tui::StateL;
use crossterm::cursor;
use crossterm::event::{Event, KeyCode, KeyEventKind};
use crossterm::{
    cursor::{MoveLeft, MoveRight, RestorePosition, SavePosition},
    ExecutableCommand,
};
use ratatui::{
    layout::{Constraint, Layout, Margin},
    style::{Style, Stylize},
    text::{Line, Span, Text},
    widgets::{
        Block, List, ListDirection, ListItem, ListState, Padding, Paragraph, Scrollbar,
        ScrollbarOrientation, ScrollbarState,
    },
    Frame,
};
use std::{io::Write, process::Command};
use Constraint::{Length, Min};

#[derive(Default)]
struct Cmd {
    cmd: String,
    args: Vec<String>,
}

impl Cmd {
    fn new(cmd: String, args: Vec<String>) -> Self {
        Self { cmd, args }
    }

    fn from_str(cmd_str: String) -> Option<Self> {
        let mut parts = cmd_str.split(' ');
        let cmd = parts.next().map(|s| s.to_string());
        let args: Vec<String> = parts.map(|s| s.to_string()).collect();
        if let Some(c) = cmd {
            return Some(Self { cmd: c, args });
        }
        return None;
    }

    fn to_string(&self) -> String {
        let cmd_str = format!("{} {}", self.cmd, self.args.join(" "));
        return cmd_str;
    }
}

pub enum GiermError {
    NotFoundError,
    CmdExecError,
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
            crate::args::Command::CLONE => {
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

                    // let cmd = Command::new("git").arg("clone").arg(url);
                    let mut args: Vec<String> = Vec::new();
                    args.push("clone".to_string());
                    args.push(url);
                    let cmd = Cmd::new("git".to_string(), args);
                    return Some(cmd);
                }
                return None;
            }
            _ => return None,
        }
    }

    async fn handle_events(&mut self) -> std::io::Result<(bool, Option<Cmd>)> {
        match crossterm::event::read()? {
            Event::Key(key) if key.kind == KeyEventKind::Press => match key.code {
                KeyCode::Esc => return Ok((true, None)),
                KeyCode::Up => self.list.state.next(), // move list up
                KeyCode::Down => self.list.state.previous(), // move list down
                KeyCode::Left => {}                    // move filter left
                KeyCode::Right => {}                   // move filter right
                KeyCode::Enter => {
                    let cmd = self.get_command();
                    return Ok((true, cmd));
                }
                KeyCode::Backspace => self.list.filter_remove_last(), // remove char from filter
                KeyCode::Tab => {} // switch filter mode to search diff user
                KeyCode::Char(c) => {
                    self.list.filter_append(c);
                }
                _ => {}
            },
            _ => {}
        }
        Ok((false, None))
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

fn exec_command(cmd: Cmd) -> Result<String, GiermError> {
    let mut command = Command::new(cmd.cmd);
    for arg in cmd.args.iter() {
        command.arg(arg);
    }

    match command.output() {
        Ok(output) => {
            if output.stderr.is_empty() {
                let out: String = String::from_utf8(output.stdout).unwrap();
                return Ok(out);
            } else {
                let err: String = String::from_utf8(output.stderr).unwrap();
                return Ok(err);
            }
        }
        Err(e) => {
            // return Err(e);
            return Err(GiermError::CmdExecError);
        }
    }
}

fn ask_confirmation(prompt: String, input_beg: &String) -> std::io::Result<(bool, String)> {
    crossterm::terminal::enable_raw_mode()?;
    let mut cur_offset: usize = 0;
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
                    if cur_offset < input.len() {
                        cur_offset += 1;
                        cout.execute(MoveLeft(1))?;
                    }
                }
                KeyCode::Right => {
                    if cur_offset > 0 {
                        cur_offset -= 1;
                        cout.execute(MoveRight(1))?;
                    }
                }
                KeyCode::Enter => {
                    crossterm::terminal::disable_raw_mode()?;
                    println!();
                    return Ok((true, input));
                }
                KeyCode::Backspace => {
                    let i: i8 = input.len() as i8 - cur_offset as i8 - 1;
                    if i >= 0 {
                        input.remove(i as usize);
                        cout.execute(MoveLeft(1))?;
                    }
                }
                KeyCode::Char(c) => {
                    cout.execute(MoveRight(1))?;
                    input.insert(input.len() - cur_offset, c);
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

pub async fn run_list_selector(
    user: crate::git::User,
    username: String,
    filter: String,
    command: crate::args::Command,
) -> Result<(), GiermError> {
    let mut list_tui: ListSearchTui;
    if username.is_empty() || username == user.git.username {
        let all_repos: Vec<String> = user.git.repos.keys().cloned().collect();
        let fl = FilterList::new(all_repos, filter);
        list_tui = ListSearchTui::new(user, None, command, fl);
    } else if let Some(mut git_user) = crate::api::search_gituser(&user, &username).await {
        let all_repos: Vec<String> = git_user.repos.keys().cloned().collect();
        let fl = FilterList::new(all_repos, filter);
        list_tui = ListSearchTui::new(user, Some(git_user), command, fl);
    } else {
        return Err(GiermError::NotFoundError);
    }

    let cmd = list_tui.run().await;
    if let Some(command) = cmd {
        let cmd_str = command.to_string();
        let input_res = ask_confirmation("Enter file path:".to_string(), &cmd_str);
        match input_res {
            Ok((true, input)) => {
                if let Ok(out) = exec_command(
                    Cmd::from_str(format!("{} {}", cmd_str, input.trim()).trim().to_string())
                        .unwrap_or_default(),
                ) {
                    println!("{}", out);
                }
            }
            _ => {}
        }
    }
    return Ok(());
}
