use std::error::Error;

use crate::git;
use reqwest;

const API_URL: &str = "https://api.github.com";

pub fn fetch_user(user: git::User) {
    let url = format!("{}/users/{}", API_URL, user.name);
}

pub fn fetch_data(url: &str) -> Result<(), reqwest::Error> {
    match reqwest::blocking::get(url) {
        Ok(res) => {
            // let body: serde_json::Value = res.json()?;
            let text = res.text()?;
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
