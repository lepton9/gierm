mod api;
mod git;
mod tui;

struct CLATypeError;

struct CLArgs {
    pub command: Option<String>,
    pub username: Option<String>,
    pub repo: Option<String>,
}

impl CLArgs {
    fn new() -> Self {
        Self {
            command: None,
            username: None,
            repo: None,
        }
    }
}

fn get_cl_args() -> Result<CLArgs, CLATypeError> {
    let args: Vec<String> = std::env::args().collect();
    let cl_args = CLArgs::new();

    for (i, arg) in args.iter().skip(1).enumerate() {
        match arg.as_str() {
            "-u" | "--user" => {
                println!("{arg} found");
            }
            "-r" | "--repo" => {
                println!("{arg} found");
            }
            "-h" | "--help" => {
                println!("Print help");
            }
            _ if arg.starts_with("-") => {
                println!("Unknown option: {arg}");
            }
            _ => {
                println!(
                    "{}: '{arg}' is not a gierm command. See 'gierm --help'.",
                    args[0]
                );
                return Err(CLATypeError);
            }
        }
    }
    return Ok(cl_args);
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args_res = get_cl_args();
    match args_res {
        Ok(args) => {}
        Err(e) => {
            std::process::exit(0);
        }
    }

    dotenv::dotenv().ok();
    let gh_access_token = std::env::var("GITHUB_ACCESS_TOKEN").expect("Set GITHUB_ACCESS_TOKEN");
    let username: String = "lepton9".to_string(); // TODO: from env or config
    let mut user: git::User = git::User::new(username, gh_access_token.to_string());

    println!("Fetching data..");
    api::fetch_user(&mut user).await;
    user.git.repos = api::fetch_repos(&user, &user.git.username).await;

    // let repo_name = "gierm".to_string();
    // let repo = api::fetch_repo(&user, &user.git.username, &repo_name)
    //     .await
    //     .unwrap();
    // let commits: Vec<git::Commit> = api::fetch_repo_commits(&user, &repo).await;
    // if let Some(repo) = user.git.repos.get_mut(&repo_name) {
    //     repo.commits = commits;
    // }

    println!("{:?}", user);

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

    tui::run_tui(user).await;

    return Ok(());
}
