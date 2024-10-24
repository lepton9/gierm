mod api;
mod git;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv::dotenv().ok();
    let gh_access_token = std::env::var("GITHUB_ACCESS_TOKEN").expect("Set GITHUB_ACCESS_TOKEN");
    let user = git::User::new("lepton9".to_string(), gh_access_token.to_string());

    println!("Fetching data..");

    // api::fetch_user(&user).await;
    api::fetch_repos(&user).await;
    // let commits: Vec<git::Commit> = api::fetch_repo_commits(&user, "gierm".to_string()).await;
    // for commit in commits {
    //     println!("{:?}", commit);
    //     api::fetch_commit_info(&user, "gierm".to_string(), &commit).await;
    // }

    // TODO: in User struct and reduce when making a request
    let rateLimit = api::fetch_data("https://api.github.com/rate_limit", &user).await;
    match rateLimit {
        Ok(v) => println!("Rate remaining: {}", v["rate"]["remaining"]),
        Err(e) => println!("Error: {:?}", e),
    }

    Ok(())
}
