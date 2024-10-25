use api::search_user;
use git::get_clone_url;

mod api;
mod git;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv::dotenv().ok();
    let gh_access_token = std::env::var("GITHUB_ACCESS_TOKEN").expect("Set GITHUB_ACCESS_TOKEN");
    let username: String = "lepton9".to_string(); // TODO: from env or config
    let mut user: git::User = git::User::new(username, gh_access_token.to_string());

    println!("Fetching data..");

    api::fetch_user(&mut user).await;
    user.repos = api::fetch_repos(&user, &user.username).await;

    let repo_name = "gierm".to_string();
    let commits: Vec<git::Commit> = api::fetch_repo_commits(&user, repo_name.clone()).await;
    if let Some(repo) = user.repos.get_mut(&repo_name) {
        repo.commits = commits;
    }
    println!("{:?}", user);

    search_user(&user, &"thePrimeagen".to_string()).await;

    // for commit in commits {
    //     println!("{:?}", commit);
    //     api::fetch_commit_info(&user, "gierm".to_string(), &commit).await;
    // }

    Ok(())
}
