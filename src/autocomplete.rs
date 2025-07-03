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

    fn is_directory(path: &str) -> bool {
        fs::metadata(path).map(|m| m.is_dir()).unwrap_or(false)
    }

    pub fn update_matches(&mut self) {
        self.matches.clear();
        let (dir, partial) = AutoComplete::split_input(&self.input);
        if partial == "." || partial == ".." {
            self.matches.push(partial.to_string());
            if partial == "." {
                self.matches.push("..".to_string());
            }
        }
        if let Ok(entries) = fs::read_dir(dir) {
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

    fn update_input_with_match(&mut self, one_match: &str) {
        let new_input = format!("{}{}", AutoComplete::split_input(&self.input).0, one_match);
        if AutoComplete::is_directory(&new_input) {
            self.input = format!("{}/", new_input);
        } else {
            self.input = new_input;
        }
    }

    pub fn complete(&mut self) -> Option<bool> {
        self.update_matches();
        match self.matches.as_slice() {
            [] => None,
            [one_match] => {
                self.update_input_with_match(&one_match.clone());
                return Some(true);
            }
            _ => Some(false),
        }
    }

    pub fn print_matches(&self) {
        println!("Matches: {:?}", self.matches);
    }
}
