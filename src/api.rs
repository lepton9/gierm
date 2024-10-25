use crate::git;
use reqwest::{self, header::HeaderMap};
use std::collections::HashMap;

const API_URL: &str = "https://api.github.com";

// TODO: fill the user struct and make a func to fetch other users by username
pub async fn fetch_user(user: &mut git::User) {
    let url = format!("{}/users/{}", API_URL, user.username);
    let res = fetch_data(&url, &user).await;
    match res {
        Ok(v) => {
            fetch_rate(user).await;
        }
        Err(e) => println!("Error: {:?}", e),
    }
}

pub async fn search_user(user: &git::User, username: &String) {
    let url = format!("{}/users/{}", API_URL, username);
    let res = fetch_data(&url, &user).await;
    match res {
        Ok(v) => {
            let repos: HashMap<String, git::Repo> = fetch_repos(user, username).await;
            // println!("{:?}", repos);
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

pub async fn fetch_repos(user: &git::User, username: &String) -> HashMap<String, git::Repo> {
    let url = format!("{}/users/{}/repos", API_URL, username);
    let res = fetch_data(&url, &user).await;
    // println!("{:?}\n", res);
    match res {
        Ok(v) => {
            let mut all_repos: HashMap<String, git::Repo> = HashMap::new();
            if let serde_json::Value::Array(repos) = &v {
                for (i, r) in repos.iter().enumerate() {
                    let repo: git::Repo = git::Repo::new(
                        r["owner"]["login"].to_string().replace("\"", ""),
                        r["name"].to_string().replace("\"", ""),
                        r["description"].to_string().replace("\"", ""),
                        r["language"].to_string().replace("\"", ""),
                    );
                    // println!("{:?}", r);
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

fn extract_next_url(link_header: &reqwest::header::HeaderValue) -> Option<String> {
    let link_str = link_header.to_str().ok()?;
    let next_pattern = r#"<([^>]+)>; rel="next""#;

    let regex = regex::Regex::new(next_pattern).ok()?;
    if let Some(captures) = regex.captures(link_str) {
        captures.get(1).map(|m| m.as_str().to_string())
    } else {
        None
    }
}

pub async fn fetch_data(
    url: &str,
    user: &git::User,
) -> Result<serde_json::Value, Box<dyn std::error::Error>> {
    // if !user.fetch() {
    //     println!("Error: Rate limit reached");
    //     return Err("Rate limit reached".into());
    // }
    let per_page = 100;
    let client = reqwest::Client::new();

    let mut fetch_url = url.to_string();
    let mut data: serde_json::Value = serde_json::Value::Null;
    loop {
        let mut headers = HeaderMap::new();
        headers.insert("User-Agent", "gierm".parse().unwrap());
        headers.insert("Accept", "application/vnd.github+json".parse().unwrap());
        headers.insert(
            "Authorization",
            format!("Token {}", user.get_token()).parse().unwrap(),
        );
        println!("GET: {}", fetch_url);
        let res = client
            .get(fetch_url)
            .headers(headers)
            .query(&[("per_page", &per_page.to_string())])
            .send()
            .await;
        match res {
            Ok(r) => {
                // let body: serde_json::Value = res.json()?;
                let link_header = r.headers().get("link").map(|h| h.to_owned());
                let text = r.text().await?;
                let body: Result<serde_json::Value, serde_json::Error> =
                    serde_json::from_str(&text);
                match body {
                    Ok(v) => {
                        // println!("Data: {:?}", v);
                        // return Ok(v);
                        match &mut data {
                            serde_json::Value::Null => {
                                data = v;
                            }
                            serde_json::Value::Array(ref mut items) => {
                                if let serde_json::Value::Array(page_items) = v {
                                    items.extend(page_items);
                                    println!("Length: {:?}", items.len());
                                }
                            }
                            serde_json::Value::Object(ref mut map) => {
                                if let serde_json::Value::Object(page_object) = v {
                                    for (k, v) in page_object {
                                        map.insert(k, v);
                                    }
                                } else {
                                    return Err("Expected an object".into());
                                }
                            }
                            _ => {
                                return Err("Unexpected data type".into());
                            }
                        }
                    }
                    Err(e) => {
                        // println!("Error JSON: {:?}", e);
                        return Err(Box::new(e));
                    }
                }
                if let Some(link_header_value) = link_header {
                    if let Some(next_url) = extract_next_url(&link_header_value) {
                        fetch_url = next_url;
                    } else {
                        return Ok(data); // No more pages
                    }
                } else {
                    return Ok(data); // No "Link" header, no more pages
                }
            }
            Err(e) => {
                // println!("Error req: {:?}", e);
                return Err(Box::new(e));
            }
        }
    }
}
