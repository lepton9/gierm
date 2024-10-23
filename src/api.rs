use std::error::Error;

use crate::git;
use reqwest::{self, header::HeaderMap};

const API_URL: &str = "https://api.github.com";

pub fn fetch_user(user: git::User) {
    let url = format!("{}/users/{}", API_URL, user.name);
}

pub async fn fetch_data(url: &str) -> Result<(), reqwest::Error> {
    let client = reqwest::Client::new();
    let mut headers = HeaderMap::new();
    headers.insert("User-Agent", "gierm".parse().unwrap());
    headers.insert("Accept", "application/vnd.github+json".parse().unwrap());
    // let res = client.get(url).headers(headers).send().await?;

    match client.get(url).headers(headers).send().await {
        // match reqwest::blocking::get(url) {
        Ok(r) => {
            // let body: serde_json::Value = res.json()?;
            let text = r.text().await?;
            let body: Result<serde_json::Value, serde_json::Error> = serde_json::from_str(&text);
            match body {
                Ok(v) => {
                    println!("Data: {:?}", v);
                }
                Err(e) => {
                    println!("Error JSON: {:?}", e);
                }
            }
        }
        Err(err) => {
            println!("Error req: {:?}", err);
        }
    }
    Ok(())
}
