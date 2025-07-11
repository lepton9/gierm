use std::process::{Command, Stdio};

pub enum CmdType {
    CLONE,
    DEFAULT,
}

pub enum CmdError {
    CmdExecError,
}

pub fn command_type(cmd: &String) -> Option<CmdType> {
    return match cmd.as_str() {
        "clone" => Some(CmdType::CLONE),
        _ => None,
    };
}

pub struct Cmd {
    cmd_type: CmdType,
    pub cmd: String,
    pub args: Vec<String>,
}

impl Cmd {
    pub fn new(cmd: String, args: Vec<String>) -> Self {
        Self {
            cmd_type: CmdType::DEFAULT,
            cmd,
            args,
        }
    }

    pub fn new_git_cmd(cmd_type: CmdType) -> Self {
        let cmd_arg: String = match cmd_type {
            CmdType::CLONE => "clone".to_string(),
            CmdType::DEFAULT => "".to_string(),
        };
        let mut args: Vec<String> = Vec::new();
        args.push(cmd_arg);
        Self {
            cmd_type,
            cmd: "git".to_string(),
            args,
        }
    }

    pub fn set_args(&mut self, args: Vec<String>) {
        self.args = args;
    }

    pub fn push_arg(&mut self, arg: String) {
        if !arg.is_empty() {
            self.args.push(arg);
        }
    }

    pub fn from_str(cmd_str: String) -> Option<Self> {
        let mut parts = cmd_str.split(' ');
        let cmd = parts.next().map(|s| s.to_string());
        let args: Vec<String> = parts.map(|s| s.to_string()).collect();
        if let Some(c) = cmd {
            return Some(Cmd::new(c, args));
        }
        return None;
    }

    pub fn to_string(&self) -> String {
        let cmd_str = format!("{} {}", self.cmd, self.args.join(" "));
        return cmd_str;
    }

    pub fn exec(&self, capture_output: bool) -> Result<String, (CmdError, String)> {
        let mut command = Command::new(self.cmd.clone());
        self.args.iter().for_each(|arg| {
            command.arg(arg);
        });
        if !capture_output {
            command.stdout(Stdio::inherit()).stderr(Stdio::inherit());
        }
        match command.output() {
            Ok(output) => {
                let stdout = String::from_utf8_lossy(&output.stdout).to_string();
                let stderr = String::from_utf8_lossy(&output.stderr).to_string();
                if output.status.success() {
                    if stdout.is_empty() {
                        Ok(stderr)
                    } else {
                        Ok(stdout)
                    }
                } else {
                    Err((CmdError::CmdExecError, stderr))
                }
            }
            Err(e) => Err((CmdError::CmdExecError, e.to_string())),
        }
    }
}
