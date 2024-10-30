mod api;
mod git;
mod tui;

enum GiermError {
    CLATypeError,
    Help,
}

const COMMANDS: [&str; 1] = ["clone"];

#[derive(Debug)]
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

fn get_cl_args() -> Result<CLArgs, GiermError> {
    let args: Vec<String> = std::env::args().collect();
    let mut cl_args = CLArgs::new();
    let mut arg_iter = args.iter().skip(1).peekable();

    if let Some(a) = arg_iter.peek() {
        if !a.starts_with("-") {
            cl_args.command = Some(a.to_string());
            arg_iter.next();

            if !COMMANDS.contains(&cl_args.command.as_ref().unwrap().as_ref()) {
                println!(
                    "{}: '{}' is not a gierm command. See 'gierm --help'.",
                    args[0],
                    cl_args.command.unwrap()
                );
                return Err(GiermError::CLATypeError);
            }
        }
    }

    while let Some(arg) = arg_iter.next() {
        match arg.as_str() {
            "-u" | "--user" => {
                if let Some(a) = arg_iter.peek() {
                    if !a.starts_with("-") {
                        if cl_args.username.is_none() {
                            cl_args.username = Some(a.to_string());
                        }
                        arg_iter.next();
                    }
                }
            }
            "-r" | "--repo" => {
                if let Some(a) = arg_iter.peek() {
                    if !a.starts_with("-") {
                        if cl_args.repo.is_none() {
                            cl_args.repo = Some(a.to_string());
                        }
                        arg_iter.next();
                    }
                }
            }
            "-h" | "--help" => {
                return Err(GiermError::Help);
            }
            _ if arg.starts_with("-") => {
                println!("Unknown option: {arg}");
                return Err(GiermError::CLATypeError);
            }
            _ => {
                println!(
                    "{}: '{arg}' is not a gierm command. See 'gierm --help'.",
                    args[0]
                );
                return Err(GiermError::CLATypeError);
            }
        }
    }
    return Ok(cl_args);
}

async fn clone(user: git::User, args: &CLArgs) {
    if args.username.is_none() || args.username.as_ref().unwrap().clone() == user.git.username {
        println!("Choose your own repo");
    } else {
        println!("Choose repo from user {}", args.username.as_ref().unwrap());
        tui::run_list_selector(
            user,
            args.username.clone().unwrap_or("".to_string()),
            args.repo.clone().unwrap_or("".to_string()),
        )
        .await;
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv::dotenv().ok();
    let args_res = get_cl_args();
    let args: CLArgs = match args_res {
        Ok(args) => args,
        Err(GiermError::Help) => {
            println!("Print help");
            return Ok(());
        }
        Err(GiermError::CLATypeError) => {
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
        match cmd.as_str() {
            "clone" => {
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
