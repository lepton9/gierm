use std::{
    fs,
    io::{self, Write},
};

#[derive(Debug)]
pub struct Match {
    pub dir: bool,
    pub name: String,
}

impl Match {
    pub fn new(name: String, dir: bool) -> Self {
        Self { name, dir }
    }
}

#[derive(Debug)]
pub struct AutoComplete {
    pub input: String,
    pub cur_path: String,
    pub matches: Vec<Match>,
    pub selected_match: Option<usize>,
}

impl AutoComplete {
    pub fn new() -> Self {
        Self {
            input: "".to_string(),
            cur_path: "".to_string(),
            matches: Vec::new(),
            selected_match: None,
        }
    }

    pub fn update_input(&mut self, input: String) {
        self.input = input;
    }

    pub fn add_char(&mut self, c: char) {
        self.input.push(c);
    }

    pub fn valid_path(path: &String) -> bool {
        fs::metadata(path).is_ok()
    }

    fn is_directory(path: &str) -> bool {
        fs::metadata(path).map(|m| m.is_dir()).unwrap_or(false)
    }

    pub fn selected(&self) -> Option<&Match> {
        return self.selected_match.and_then(|i| self.matches.get(i));
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

    pub fn update_matches(&mut self) {
        self.matches.clear();
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
        let new_input = format!("{}{}", path, m.name);
        if m.dir {
            return format!("{}/", new_input);
        } else {
            return new_input;
        }
    }

    pub fn complete(&mut self) -> Option<bool> {
        self.update_matches();
        match self.matches.as_slice() {
            [] => None,
            [one_match] => {
                self.input = AutoComplete::completed_input(self.inputted_dir(), one_match);
                return Some(true);
            }
            _ => Some(false),
        }
    }

    pub fn print_matches(&self) {
        println!("Matches: {:?}", self.matches);
    }
}
