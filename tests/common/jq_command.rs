use assert_cmd::assert::Assert;
use assert_cmd::Command;

pub struct JqCommand {
    cmd: Command,
    stdin_data: Option<String>,
}

impl JqCommand {
    pub fn new() -> Self {
        let cmd = Command::cargo_bin("jq").expect("jq binary not found");
        Self {
            cmd,
            stdin_data: None,
        }
    }

    pub fn arg<S: AsRef<str>>(mut self, arg: S) -> Self {
        self.cmd.arg(arg.as_ref());
        self
    }

    pub fn args<I, S>(mut self, args: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: AsRef<str>,
    {
        for arg in args {
            self.cmd.arg(arg.as_ref());
        }
        self
    }

    pub fn stdin(mut self, input: &str) -> Self {
        self.stdin_data = Some(input.to_string());
        self
    }

    pub fn assert(mut self) -> Assert {
        if let Some(input) = self.stdin_data {
            self.cmd.write_stdin(input);
        }
        self.cmd.assert()
    }
}
