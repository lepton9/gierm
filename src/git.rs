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
    commits: Vec<Commit>,
}

// TODO: tree, files modified
#[derive(Debug)]
pub struct Commit {
    pub message: String,
    pub sha: String,
    pub committer: String, // Username
    pub date: DateTime<Utc>,
    // pub info: CommitInfo,
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

#[derive(Debug)]
pub struct CommitInfo {
    pub total_changes: i32,
    pub additions: i32,
    pub deletions: i32,
    pub files: Vec<File>,
}

impl CommitInfo {
    pub fn new(total_changes: i32, additions: i32, deletions: i32) -> Self {
        Self {
            total_changes,
            additions,
            deletions,
            files: Vec::new(),
        }
    }
    pub fn default() -> Self {
        Self {
            total_changes: 0,
            additions: 0,
            deletions: 0,
            files: Vec::new(),
        }
    }
}

#[derive(Debug)]
pub struct File {
    pub name: String,
    pub sha: String,
    pub additions: i32,
    pub deletions: i32,
}

impl File {
    pub fn new(name: String, sha: String, additions: i32, deletions: i32) -> Self {
        Self {
            name,
            sha,
            additions,
            deletions,
        }
    }
}
