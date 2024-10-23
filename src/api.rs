use std::error::Error;

use crate::git;
use reqwest::{self, header::HeaderMap};

const API_URL: &str = "https://api.github.com";

pub fn fetch_user(user: git::User) {
    let url = format!("{}/users/{}", API_URL, user.username);
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
