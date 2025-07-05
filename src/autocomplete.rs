use std::fs;

pub enum CompletionError {
    NoSelectedMatch,
    NoMatches,
}

#[derive(Debug)]
pub struct Match {
    dir: bool,
    name: String,
}

impl Match {
    pub fn new(name: String, dir: bool) -> Self {
        Self { name, dir }
    }

    pub fn to_string(&self) -> String {
        if self.dir {
            return format!("{}/", self.name);
        }
        return self.name.clone();
    }
}

#[derive(Debug)]
pub struct AutoComplete {
    input: String,
    input_changed: bool,
    cur_path: String,
    matches: Vec<Match>,
    selected_match: Option<usize>,
}

impl AutoComplete {
    pub fn new() -> Self {
        Self {
            input: "".to_string(),
            input_changed: true,
            cur_path: "".to_string(),
            matches: Vec::new(),
            selected_match: None,
        }
    }

    pub fn update_input(&mut self, input: String) {
        self.input = input;
        self.input_changed = true;
    }

    pub fn add_char(&mut self, c: char) {
        self.input.push(c);
        self.input_changed = true;
    }

    pub fn delete_char(&mut self) {
        self.input.pop();
        self.input_changed = true;
    }

    pub fn get_input(&self) -> String {
        return self.input.clone();
    }

    pub fn get_matches(&self) -> Vec<String> {
        return self.matches.iter().map(|m| m.to_string()).collect();
    }

    fn valid_path(path: &String) -> bool {
        fs::metadata(path).is_ok()
    }

    fn is_directory(path: &str) -> bool {
        fs::metadata(path).map(|m| m.is_dir()).unwrap_or(false)
    }

    pub fn clear_matches(&mut self) {
        self.matches.clear();
        self.selected_match = None;
    }

    pub fn selected(&self) -> Option<&Match> {
        return self.selected_match.and_then(|i| self.matches.get(i));
    }

    pub fn selected_index(&self) -> Option<usize> {
        return self.selected_match;
    }

    pub fn select_next(&mut self) {
        if self.matches.is_empty() {
            self.selected_match = None;
        } else {
            self.selected_match = Some(
                self.selected_match
                    .map_or(0, |i| (i + 1) % self.matches.len()),
            );
        }
    }

    fn update_matches(&mut self) {
        self.clear_matches();
        let (dir, partial) = AutoComplete::split_input(&self.input);
        if partial == "." || partial == ".." {
            self.matches.push(Match::new(partial.to_string(), true));
            if partial == "." {
                self.matches.push(Match::new("..".to_string(), true));
            }
        }
        if let Ok(entries) = fs::read_dir(dir) {
            for entry in entries {
                if let Ok(entry) = entry {
                    let file_name = entry.file_name();
                    let file_name_str = file_name.to_string_lossy();
                    if file_name_str.starts_with(&partial) {
                        self.matches.push(Match::new(
                            file_name_str.to_string(),
                            AutoComplete::is_directory(&self.add_to_cur_path(&file_name_str)),
                        ));
                    }
                }
            }
        }
    }

    fn split_input(input: &String) -> (&str, &str) {
        if let Some(pos) = input.rfind('/') {
            let (dir, partial) = input.split_at(pos + 1);
            (dir, partial)
        } else {
            ("./", &input)
        }
    }

    fn inputted_dir(&self) -> &str {
        return AutoComplete::split_input(&self.input).0;
    }

    fn add_to_cur_path(&self, file: &str) -> String {
        return format!("{}{}", self.inputted_dir(), file);
    }

    fn completed_input(path: &str, m: &Match) -> String {
        return format!("{}{}", path, m.to_string());
    }

    pub fn input_with_match(&self, m: &Match) -> String {
        return AutoComplete::completed_input(self.inputted_dir(), m);
    }

    pub fn accept_selected_match(&mut self) -> Result<(), CompletionError> {
        let m = self.selected().ok_or(CompletionError::NoSelectedMatch)?;
        self.update_input(self.input_with_match(m));
        self.input_changed = true;
        self.clear_matches();
        return Ok(());
    }

    pub fn complete(&mut self) -> Option<bool> {
        let updated = if self.input_changed {
            self.input_changed = false;
            self.update_matches();
            true
        } else {
            false
        };
        match self.matches.as_slice() {
            [] => {
                self.clear_matches();
                return None;
            }
            [one_match] => {
                self.update_input(self.input_with_match(one_match));
                self.clear_matches();
                return Some(true);
            }
            _ => {
                if !updated {
                    self.select_next();
                }
                return Some(false);
            }
        }
    }

    pub fn print_matches(&self) {
        println!("Matches: {:?}", self.matches);
    }
}
