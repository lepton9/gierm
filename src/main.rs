mod api;
mod git;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv::dotenv().ok();
    let gh_access_token = std::env::var("GITHUB_ACCESS_TOKEN").expect("Set GITHUB_ACCESS_TOKEN");
    let user = git::User::new("lepton9".to_string(), gh_access_token.to_string());

    println!("Fetching data..");
    // let res = api::fetch_data("https://api.github.com/users/lepton9").await?;
    // let res = api::fetch_data("https://api.github.com/repos/lepton9/lim/actions/caches").await?;
    let rateLimit = api::fetch_data("https://api.github.com/rate_limit", &user).await;
    match rateLimit {
        Ok(v) => println!("Rate remaining: {}", v["rate"]["remaining"]),
        Err(e) => println!("Error: {:?}", e),
    }

    Ok(())
}
