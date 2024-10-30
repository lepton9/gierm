mod api;
mod args;
mod git;
mod tui;

async fn clone(user: git::User, args: &args::CLArgs) {
    if args.username.is_none() || args.username.as_ref().unwrap().clone() == user.git.username {
        println!("Choose your own repo");
        let res = tui::run_list_selector(
            user,
            "".to_string(),
            args.repo.clone().unwrap_or("".to_string()),
            args::Command::CLONE,
        )
        .await;
    } else {
        println!("Choose repo from user {}", args.username.as_ref().unwrap());
        let res = tui::run_list_selector(
            user,
            args.username.clone().unwrap_or("".to_string()),
            args.repo.clone().unwrap_or("".to_string()),
            args::Command::CLONE,
        )
        .await;
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv::dotenv().ok();
    let args_res = args::get_cl_args();
    let args: args::CLArgs = match args_res {
        Ok(args) => args,
        Err(args::ArgsError::Help) => {
            args::help();
            return Ok(());
        }
        Err(args::ArgsError::CLATypeError) => {
            println!("Error");
            return Ok(());
        }
    };
    println!("{:?}", args);

    let gh_access_token = std::env::var("GITHUB_ACCESS_TOKEN").expect("Set GITHUB_ACCESS_TOKEN");
    let username: String = "lepton9".to_string(); // TODO: from env or config
    let mut user: git::User = git::User::new(username, gh_access_token.to_string());

    api::fetch_user(&mut user).await;
    user.git.repos = api::fetch_repos(&user, &user.git.username).await;

    if let Some(cmd) = &args.command {
        match args::command(cmd) {
            Some(args::Command::CLONE) => {
                clone(user, &args).await;
                return Ok(());
            }
            _ => {}
        }
    }
    tui::run_tui(user).await;

    // let repo_name = "gierm".to_string();
    // let repo = api::fetch_repo(&user, &user.git.username, &repo_name)
    //     .await
    //     .unwrap();
    // let commits: Vec<git::Commit> = api::fetch_repo_commits(&user, &repo).await;
    // if let Some(repo) = user.git.repos.get_mut(&repo_name) {
    //     repo.commits = commits;
    // }

    // println!("{:?}", user);

    // if let Some(repo) = user.repos.get(&repo_name) {
    //     for commit in &repo.commits {
    //         api::fetch_commit_info(&user, user.username.clone(), repo.name.clone(), commit).await;
    //         println!("{:?}", commit);
    //     }
    // }

    // let git_user: git::GitUser = api::search_user(&user, &"thePrimeagen".to_string())
    //     .await
    //     .unwrap();
    // println!("{:?}", git_user);

    return Ok(());
}
