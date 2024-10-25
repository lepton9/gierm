use std::collections::HashMap;

use crate::git;
use reqwest::{self, header::HeaderMap};

const API_URL: &str = "https://api.github.com";

// TODO: fill the user struct and make a func to fetch other users by username
pub async fn fetch_user(user: &mut git::User) {
    let url = format!("{}/users/lepton9", API_URL);
    // let url = format!("{}/users/{}", API_URL, user.username);
    let res = fetch_data(&url, &user).await;
    match res {
        Ok(v) => {
            fetch_rate(user).await;
            // println!("User: {}", v);
        }
        Err(e) => println!("Error: {:?}", e),
    }
}

pub async fn fetch_rate(user: &mut git::User) {
    match fetch_data(&format!("{}/rate_limit", API_URL), &user).await {
        Ok(v) => {
            let rate_limit: i32 = v["rate"]["remaining"].as_i64().unwrap_or(0) as i32;
            user.set_ratelimit(rate_limit);
            println!("Rate remaining: {}", user.rate());
        }
        Err(e) => println!("Error: {:?}", e),
    }
}

pub async fn fetch_repos(user: &git::User) -> HashMap<String, git::Repo> {
    let url = format!("{}/users/{}/repos", API_URL, user.username);
    let res = fetch_data(&url, &user).await;
    match res {
        Ok(v) => {
            // let mut all_repos: Vec<git::Repo> = Vec::new();
            let mut all_repos: HashMap<String, git::Repo> = HashMap::new();
            if let serde_json::Value::Array(repos) = &v {
                for (i, r) in repos.iter().enumerate() {
                    let repo: git::Repo = git::Repo::new(
                        r["owner"]["login"].to_string().replace("\"", ""),
                        r["name"].to_string().replace("\"", ""),
                        r["description"].to_string().replace("\"", ""),
                        r["language"].to_string().replace("\"", ""),
                    );
                    all_repos.insert(repo.name.clone(), repo);
                }
            }
            return all_repos;
        }
        Err(e) => {
            println!("Error: {:?}", e);
            return HashMap::new();
        }
    }
}

pub async fn fetch_repo_commits(user: &git::User, repo_name: String) -> Vec<git::Commit> {
    let url = format!("{}/repos/{}/{}/commits", API_URL, user.username, repo_name);
    let res = fetch_data(&url, &user).await;
    match res {
        Ok(v) => {
            let mut repo_commits: Vec<git::Commit> = Vec::new();
            if let serde_json::Value::Array(commits) = v {
                for (i, c) in commits.iter().enumerate() {
                    let commit: git::Commit = git::Commit::new(
                        c["commit"]["message"].to_string().replace("\"", ""),
                        c["sha"].to_string().replace("\"", ""),
                        c["committer"]["login"].to_string().replace("\"", ""),
                        c["commit"]["author"]["date"].to_string().replace("\"", ""),
                    );
                    repo_commits.push(commit);
                    // println!("git::Commit: {:?}\n", c)
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
                let mut commit_info = git::CommitInfo::new(total, additions, deletions);

                if let serde_json::Value::Array(files) = &info["files"] {
                    for (i, f) in files.iter().enumerate() {
                        let file: git::File = git::File::new(
                            f["filename"].to_string().replace("\"", ""),
                            f["sha"].to_string().replace("\"", ""),
                            f["additions"].as_i64().unwrap_or(0) as i32,
                            f["deletions"].as_i64().unwrap_or(0) as i32,
                        );
                        commit_info.files.push(file);
                    }
                }
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

pub async fn fetch_data(
    url: &str,
    user: &git::User,
) -> Result<serde_json::Value, Box<dyn std::error::Error>> {
    // if !user.fetch() {
    //     println!("Error: Rate limit reached");
    //     return Err();
    // }
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
