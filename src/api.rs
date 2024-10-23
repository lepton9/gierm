use crate::git;
use reqwest;

const API_URL: &str = "https://api.github.com";

pub fn fetch_user(user: git::User) {
    let url = format!("{}/users/{}", API_URL, user.name);
}

pub fn fetch_data(url: &str) -> Result<(), reqwest::Error> {
    let res = reqwest::blocking::get(url)?;
    println!("status = {:?}", res.status());
    let body: serde_json::Value = res.json()?;
    println!("body = {:?}", body);

    Ok(())
}

