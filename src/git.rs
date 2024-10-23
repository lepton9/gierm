// use base64::{engine::general_purpose, Engine as _};

pub struct User {
    pub username: String,
    password: String,
    pub repos: Vec<Repo>,
}

impl User {
    pub fn new(username: String, password: String) -> Self {
        Self {
            username,
            password,
            repos: Vec::new(),
        }
    }

    pub fn get_token(&self) -> String {
        return self.password.clone();
        // return general_purpose::STANDARD.encode(&format!("{}:{}", self.username, self.password));
    }
}

pub struct Repo {
    user: User,
    name: String,
    commits: Commit,
}

pub struct Commit {
    message: String,
}
