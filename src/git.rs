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

#[derive(Debug)]
pub struct Repo {
    user: String,
    name: String,
    description: String,
    language: String,
    commits: Vec<Commit>,
}

impl Repo {
    pub fn new(user: String, name: String, description: String, language: String) -> Self {
        Self {
            user,
            name,
            description,
            language,
            commits: Vec::new(),
        }
    }
}

pub fn get_clone_url(user: String, repo: String, ssh: bool) -> String {
    if ssh {
        return format!("git@github.com:{}/{}.git", user, repo);
    } else {
        return format!("https://github.com/{}/{}.git", user, repo);
    }
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
