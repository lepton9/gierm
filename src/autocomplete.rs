use std::{
    fs,
    io::{self, Write},
};

#[derive(Debug)]
pub struct AutoComplete {
    pub input: String,
    pub cur_path: String,
    pub matches: Vec<String>,
}

impl AutoComplete {
    pub fn new() -> Self {
        Self {
            input: "".to_string(),
            cur_path: "".to_string(),
            matches: Vec::new(),
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

    pub fn update_matches(&mut self) {
        let (dir, partial) = AutoComplete::split_input(&self.input);

        if let Ok(entries) = fs::read_dir(dir) {
            self.matches.clear();
            for entry in entries {
                if let Ok(entry) = entry {
                    let file_name = entry.file_name();
                    let file_name_str = file_name.to_string_lossy();

                    if file_name_str.starts_with(&partial) {
                        self.matches.push(file_name_str.to_string());
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

    pub fn complete(&mut self) -> Option<bool> {
        self.update_matches();
        match self.matches.len() {
            0 => None,
            1 => {
                self.input = format!(
                    "{}{}",
                    AutoComplete::split_input(&self.input).0,
                    self.matches[0]
                );
                return Some(true);
            }
            _n => {
                self.print_matches();
                return Some(false);
            }
        }
    }

    pub fn print_matches(&self) {
        println!("Matches: {:?}", self.matches);
    }
}
