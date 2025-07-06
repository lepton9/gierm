pub enum CmdType {
    CLONE,
    DEFAULT,
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
        self.args.push(arg);
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
}
