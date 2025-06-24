pub enum Command {
    CLONE,
}

pub fn command(cmd: &String) -> Option<Command> {
    return match cmd.as_str() {
        "clone" => Some(Command::CLONE),
        _ => None,
    };
}

pub enum ArgsError {
    CLATypeError,
    Help,
}

#[derive(Debug)]
pub struct CLArgs {
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

pub fn get_cl_args() -> Result<CLArgs, ArgsError> {
    let args: Vec<String> = std::env::args().collect();
    let mut cl_args = CLArgs::new();
    let mut arg_iter = args.iter().skip(1).peekable();

    if let Some(a) = arg_iter.peek() {
        if !a.starts_with("-") {
            cl_args.command = Some(a.to_string());
            arg_iter.next();

            if command(cl_args.command.as_ref().unwrap_or(&"".to_string())).is_none() {
                println!(
                    "{}: '{}' is not a gierm command. See 'gierm --help'.",
                    args[0],
                    cl_args.command.unwrap_or_default()
                );
                return Err(ArgsError::CLATypeError);
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
                return Err(ArgsError::Help);
            }
            _ if arg.starts_with("-") => {
                println!("Unknown option: {arg}");
                return Err(ArgsError::CLATypeError);
            }
            _ => {
                println!(
                    "{}: '{arg}' is not a gierm command. See 'gierm --help'.",
                    args[0]
                );
                return Err(ArgsError::CLATypeError);
            }
        }
    }
    return Ok(cl_args);
}

pub fn help() {
    println!("Print help");
}
