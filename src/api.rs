use crate::git;
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
pub async fn fetch_repo_commits(user: &git::User, repo_name: String) {
    let url = format!("{}/repos/{}/{}", API_URL, user.username, repo_name);
    let res = fetch_data(&url, &user).await;
    match res {
        Ok(v) => {
            println!("Commits: {}", v)
        }
        Err(e) => println!("Error: {:?}", e),
    }
}

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
