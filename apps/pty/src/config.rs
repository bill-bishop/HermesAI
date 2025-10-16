use std::path::Path;

#[derive(Clone, Debug)]
pub struct ShellProfile {
    pub program: String,
    pub args: Vec<String>,
}

pub fn resolve_profile(name: Option<&str>) -> ShellProfile {
    let name = name.unwrap_or("default").to_ascii_lowercase();
    match name.as_str() {
        "posix" => ShellProfile { program: "/bin/sh".into(), args: vec!["-i".into()] },
        "zsh" => ShellProfile { program: "/bin/zsh".into(), args: vec!["-li".into()] },
        "busybox" => ShellProfile { program: "/bin/busybox".into(), args: vec!["sh".into(), "-i".into()] },
        _ => {
            let bash = "/bin/bash";
            if Path::new(bash).exists() {
                ShellProfile { program: bash.into(), args: vec!["-li".into()] }
            } else {
                ShellProfile { program: "/bin/sh".into(), args: vec!["-i".into()] }
            }
        }
    }
}
