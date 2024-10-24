use std::env::args;

use crate::git::{self, Commit};
use reqwest::{self, header::HeaderMap};

const API_URL: &str = "https://api.github.com";

// TODO: fill the user struct and make a func to fetch other users by username
pub async fn fetch_user(user: &git::User) {
    let url = format!("{}/users/tovialhi", API_URL);
    // let url = format!("{}/users/{}", API_URL, user.username);
    let res = fetch_data(&url, &user).await;
    match res {
        Ok(v) => println!("User: {}", v),
        Err(e) => println!("Error: {:?}", e),
    }
}

// TODO: return vec
pub async fn fetch_repos(user: &git::User) {
    let url = format!("{}/users/{}/repos", API_URL, user.username);
    let res = fetch_data(&url, &user).await;
    match res {
        Ok(v) => {
            // println!("Repos: {}", v)
            println!("{}: {}", v[0]["name"], v[0]);
        }
        Err(e) => println!("Error: {:?}", e),
    }
}

// TODO: return a vec of commit structs
pub async fn fetch_repo_commits(user: &git::User, repo_name: String) -> Vec<Commit> {
    let url = format!("{}/repos/{}/{}/commits", API_URL, user.username, repo_name);
    let res = fetch_data(&url, &user).await;
    match res {
        Ok(v) => {
            let mut repo_commits: Vec<Commit> = Vec::new();
            if let serde_json::Value::Array(commits) = v {
                for (i, c) in commits.iter().enumerate() {
                    let commit: git::Commit = git::Commit::new(
                        c["commit"]["message"].to_string().replace("\"", ""),
                        c["sha"].to_string().replace("\"", ""),
                        c["committer"]["login"].to_string().replace("\"", ""),
                        c["commit"]["author"]["date"].to_string().replace("\"", ""),
                    );
                    repo_commits.push(commit);
                    // println!("Commit: {:?}\n", c)
                }
            }
            return repo_commits;
            // println!("Commits: {}", v)
        }
        Err(e) => {
            println!("Error: {:?}", e);
            return Vec::new();
        }
    }
}

pub async fn fetch_commit_info(
    user: &git::User,
    repo_name: String,
    commit: &git::Commit,
) -> git::CommitInfo {
    let url = format!(
        "{}/repos/{}/{}/commits/{}",
        API_URL, user.username, repo_name, commit.sha
    );
    let res = fetch_data(&url, &user).await;
    match res {
        Ok(v) => {
            if let serde_json::Value::Object(info) = v {
                let total = info["stats"]["total"].as_i64().unwrap_or(0) as i32;
                let additions = info["stats"]["additions"].as_i64().unwrap_or(0) as i32;
                let deletions = info["stats"]["deletions"].as_i64().unwrap_or(0) as i32;
                let commit_info = git::CommitInfo::new(total, additions, deletions);

                println!("Info: {:?}", commit_info);
                return commit_info;
            }
            return git::CommitInfo::default();
        }
        Err(e) => {
            println!("Error: {:?}", e);
            return git::CommitInfo::default();
        }
    }
}
// {
//   "sha": "0eb809d449546e9597ea6de44fc97a2e4e7be8f4",
//   "filename": ".gitignore",
//   "status": "added",
//   "additions": 4,
//   "deletions": 0,
//   "changes": 4,
//   "blob_url": "https://github.com/lepton9/gierm/blob/9f59e30e89ca4f8f4d898c3d4127fc7f062cb3c9/.gitignore",
//   "raw_url": "https://github.com/lepton9/gierm/raw/9f59e30e89ca4f8f4d898c3d4127fc7f062cb3c9/.gitignore",
//   "contents_url": "https://api.github.com/repos/lepton9/gierm/contents/.gitignore?ref=9f59e30e89ca4f8f4d898c3d4127fc7f062cb3c9",
//   "patch": "@@ -0,0 +1,4 @@\n+/debug\n+/target\n+\n+Cargo.lock"
// },

// pub async fn fetch_data(url: &str) -> Result<serde_json::Value, reqwest::Error> {
pub async fn fetch_data(
    url: &str,
    user: &git::User,
) -> Result<serde_json::Value, Box<dyn std::error::Error>> {
    let client = reqwest::Client::new();
    let mut headers = HeaderMap::new();
    headers.insert("User-Agent", "gierm".parse().unwrap());
    headers.insert("Accept", "application/vnd.github+json".parse().unwrap());
    headers.insert(
        "Authorization",
        format!("Token {}", user.get_token()).parse().unwrap(),
    );
    // let res = client.get(url).headers(headers).send().await?;

    match client.get(url).headers(headers).send().await {
        // match reqwest::blocking::get(url) {
        Ok(r) => {
            // let body: serde_json::Value = res.json()?;
            let text = r.text().await?;
            let body: Result<serde_json::Value, serde_json::Error> = serde_json::from_str(&text);
            match body {
                Ok(v) => {
                    // println!("Data: {:?}", v);
                    return Ok(v);
                }
                Err(e) => {
                    // println!("Error JSON: {:?}", e);
                    return Err(Box::new(e));
                }
            }
        }
        Err(e) => {
            // println!("Error req: {:?}", e);
            return Err(Box::new(e));
        }
    }
    // Ok(())
}
