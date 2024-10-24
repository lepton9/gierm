use std::str::FromStr;

use chrono::{DateTime, Local, Utc};

pub struct User {
    pub username: String,
    password: String,
    email: String,
    pub repos: Vec<Repo>,
}

impl User {
    pub fn new(username: String, password: String) -> Self {
        Self {
            username,
            password,
            email: "".to_string(), // TODO: get email
            repos: Vec::new(),
        }
    }

    pub fn get_token(&self) -> String {
        return self.password.clone();
    }
}

pub struct Repo {
    user: User,
    name: String,
    commits: Commit,
}

// TODO: tree, files modified
#[derive(Debug)]
pub struct Commit {
    pub message: String,
    pub date: DateTime<Utc>,
    // pub url: String,
    pub sha: String,
    pub committer: String, // Username
}

impl Commit {
    pub fn new(message: String, sha: String, committer: String, date: String) -> Self {
        let dt_result = DateTime::parse_from_rfc3339(&date);
        let dt;
        match dt_result {
            Ok(v) => {
                dt = v.to_utc();
            }
            Err(e) => {
                dt = Utc::now();
                println!("Error parse: {:?}", e);
            }
        }
        Self {
            message,
            sha,
            committer,
            date: dt,
        }
    }
}
