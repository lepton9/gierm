use chrono::{DateTime, Utc};
use std::collections::HashMap;

#[derive(Debug)]
pub struct GitUser {
    pub username: String, // Login name
    pub name: String,
    pub email: String,
    pub bio: String,
    pub repos: HashMap<String, Repo>,
}

impl GitUser {
    pub fn new(username: String, name: String, email: String, bio: String) -> Self {
        Self {
            username,
            name,
            email,
            bio,
            repos: HashMap::new(),
        }
    }
}

#[derive(Debug)]
pub struct User {
    pub git: GitUser,
    password: String,
    remaining: i32,
}

impl User {
    pub fn new(username: String, password: String) -> Self {
        Self {
            git: GitUser::new(username, "".to_string(), "".to_string(), "".to_string()),
            password,
            remaining: 60,
        }
    }

    pub fn get_token(&self) -> String {
        return self.password.clone();
    }

    pub fn set_ratelimit(&mut self, limit: i32) {
        self.remaining = limit;
    }

    pub fn rate(&self) -> i32 {
        return self.remaining;
    }

    pub fn fetch(&mut self) -> bool {
        if self.remaining > 0 {
            self.remaining -= 1;
            return true;
        }
        return false;
    }
}

#[derive(Debug)]
pub struct Repo {
    pub user: String,
    pub name: String,
    pub description: String,
    pub language: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub commits: Vec<Commit>,
}

impl Repo {
    pub fn new(
        user: String,
        name: String,
        description: String,
        language: String,
        created_at: String,
        updated_at: String,
    ) -> Self {
        let created_result = DateTime::parse_from_rfc3339(&created_at);
        let updated_result = DateTime::parse_from_rfc3339(&updated_at);
        let created: DateTime<Utc> = created_result.unwrap_or(DateTime::default()).into();
        let updated: DateTime<Utc> = updated_result.unwrap_or(DateTime::default()).into();

        Self {
            user,
            name,
            description,
            language,
            created_at: created,
            updated_at: updated,
            commits: Vec::new(),
        }
    }
}

pub fn get_clone_url(user: &String, repo: &String, ssh: bool) -> String {
    if ssh {
        return format!("git@github.com:{}/{}.git", user, repo);
    } else {
        return format!("https://github.com/{}/{}.git", user, repo);
    }
}

#[derive(Debug)]
pub struct Commit {
    pub message: String,
    pub sha: String,
    pub committer: String, // Username
    pub date: DateTime<Utc>,
    pub info: Option<CommitInfo>,
}

impl Commit {
    pub fn new(message: String, sha: String, committer: String, date: String) -> Self {
        let dt_result = DateTime::parse_from_rfc3339(&date);
        let dt: DateTime<Utc> = dt_result.unwrap_or(DateTime::default()).into();
        Self {
            message,
            sha,
            committer,
            date: dt,
            info: None,
        }
    }

    pub fn to_string(&self) -> String {
        return format!("{} {}", self.date, self.message_short());
    }

    pub fn message_short(&self) -> String {
        let i_nl = self.message.find("\\n\\n").unwrap_or(self.message.len());
        return self.message.chars().into_iter().take(i_nl).collect();
    }

    pub fn description(&self) -> String {
        let len = "\\n\\n".len();
        match self.message.find("\\n\\n") {
            Some(i) => {
                return self.message[i + len..].to_string();
            }
            _ => return "".to_string(),
        }
    }

    pub fn sha_short(&self) -> String {
        return self.sha.chars().into_iter().take(8).collect();
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
    pub patch_diff: String,
}

impl File {
    pub fn new(
        name: String,
        sha: String,
        additions: i32,
        deletions: i32,
        patch_diff: String,
    ) -> Self {
        Self {
            name,
            sha,
            additions,
            deletions,
            patch_diff,
        }
    }
}
