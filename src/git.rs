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
pub struct Commit {
    pub message: String,
    pub date: String, // TODO: date
    pub url: String,
    pub sha: String,
    pub commiter: String, // Username
}
