mod api;
mod git;
mod tui;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv::dotenv().ok();
    let gh_access_token = std::env::var("GITHUB_ACCESS_TOKEN").expect("Set GITHUB_ACCESS_TOKEN");
    let username: String = "lepton9".to_string(); // TODO: from env or config
    let mut user: git::User = git::User::new(username, gh_access_token.to_string());

    println!("Fetching data..");

    api::fetch_user(&mut user).await;
    user.git.repos = api::fetch_repos(&user, &user.git.username).await;

    let repo_name = "gierm".to_string();
    let repo = api::fetch_repo(&user, &user.git.username, &repo_name)
        .await
        .unwrap();
    let commits: Vec<git::Commit> = api::fetch_repo_commits(&user, &repo).await;
    if let Some(repo) = user.git.repos.get_mut(&repo_name) {
        repo.commits = commits;
    }
    // println!("{:?}", user);

    // if let Some(repo) = user.repos.get(&repo_name) {
    //     for commit in &repo.commits {
    //         api::fetch_commit_info(&user, user.username.clone(), repo.name.clone(), commit).await;
    //         println!("{:?}", commit);
    //     }
    // }

    let git_user: git::GitUser = api::search_user(&user, &"thePrimeagen".to_string())
        .await
        .unwrap();
    println!("{:?}", git_user);

    return Ok(());
}
