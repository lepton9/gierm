use std::fs::File;
use std::io::{prelude::*, BufReader};

mod api;
mod args;
mod autocomplete;
mod command;
mod cursor;
mod filterlist;
mod git;
mod input;
mod layout;
mod listtui;
mod tui;

const ACCESS_TOKEN: &str = "GITHUB_ACCESS_TOKEN";
const CONFIG_FILE: &str = ".giermconfig";
const CONFIG_PATHS: [&str; 2] = ["/", "/.config/gierm/"];

#[derive(Debug)]
struct Config {
    username: Option<String>,
    password: Option<String>,
}

impl Config {
    fn new(username: Option<String>, password: Option<String>) -> Self {
        Self { username, password }
    }
}

fn find_config_file() -> Option<Config> {
    let mut file: Option<File> = None;
    for path in CONFIG_PATHS.iter() {
        match File::open(format!(
            "{}{}{}",
            std::env::var("HOME").unwrap_or("~".to_string()),
            path,
            CONFIG_FILE
        )) {
            Ok(f) => {
                file = Some(f);
                break;
            }
            Err(_) => continue,
        }
    }

    let reader = match file {
        Some(file) => BufReader::new(file),
        None => return None,
    };

    let mut config: Config = Config::new(None, None);

    for line in reader.lines() {
        match line {
            Ok(l) => {
                if l.clone().to_lowercase().starts_with("username") {
                    if let Some(eq_pos) = l.find('=') {
                        let (_, username) = l.split_at(eq_pos + 1);
                        config.username = Some(username.trim().to_string());
                    }
                } else if l.clone().to_lowercase().starts_with("password") {
                    if let Some(eq_pos) = l.find('=') {
                        let (_, password) = l.split_at(eq_pos + 1);
                        config.password = Some(password.trim().to_string());
                    }
                }
            }
            Err(_) => {}
        }
    }
    return Some(config);
}

fn save_to_file(file_path: String, data: String) {
    let mut file = File::create(&file_path).expect("File creation failed");
    file.write(data.as_bytes()).expect("File write failed");
    println!("Saved to {}", file_path);
}

async fn login_user() -> Option<git::User> {
    let config = find_config_file();

    let mut username: String = String::default();
    let mut password: String = std::env::var(ACCESS_TOKEN).unwrap_or_default();

    if password.is_empty() {
        if let Some(c) = &config {
            password = c.password.clone().unwrap_or_default();
        }
    }
    if password.is_empty() {
        println!(
            "Set '{}' as environment variable or as 'password' in {} file.",
            ACCESS_TOKEN, CONFIG_FILE
        );
        println!("The config file should be located in '~/' or '~/.config/gierm/'\n");
        println!("Get a Github personal access token from 'https://github.com/settings/tokens'");
        return None;
    }

    if let Some(c) = config {
        username = c.username.clone().unwrap_or_default();
    }

    if username.is_empty() {
        println!("No existing user. Login with Github user.");
        if let Ok((confirm, input)) =
            input::ask_input("Enter username".to_string(), &"".to_string())
        {
            if !confirm {
                return None;
            }
            username = input;
            save_to_file(
                format!(
                    "{}/{}",
                    std::env::var("HOME").unwrap_or("~".to_string()),
                    CONFIG_FILE
                ),
                format!("username={}", username),
            );
        }
    }

    let mut user: git::User = git::User::new(username, password);
    match api::fetch_user(&mut user).await {
        Ok(_) => {
            user.git.repos = api::fetch_repos(&user, &user.git.username).await;
            return Some(user);
        }
        _ => return None,
    }
}

async fn clone(user: git::User, args: &args::CLArgs) {
    let username =
        match args.username.is_none() || args.username.as_deref() == Some(&user.git.username) {
            true => "".to_string(),
            false => args.username.clone().unwrap_or_default(),
        };
    let _res = listtui::run_list_selector(
        user,
        username,
        args.repo.clone().unwrap_or_default(),
        command::CmdType::CLONE,
    )
    .await;
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

    print!("Fetching user...");
    std::io::stdout().flush().unwrap();
    let user = match login_user().await {
        Some(user) => user,
        None => {
            println!("\x1b[2K\rLogin failed..");
            return Ok(());
        }
    };
    print!("\x1b[2K\r");
    std::io::stdout().flush().unwrap();

    if let Some(cmd) = &args.command {
        match command::command_type(cmd) {
            Some(command::CmdType::CLONE) => {
                clone(user, &args).await;
                return Ok(());
            }
            _ => {}
        }
    }
    tui::run_tui(user).await;

    return Ok(());
}
