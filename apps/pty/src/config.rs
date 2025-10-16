use serde::Deserialize;

#[derive(Debug, Deserialize, Clone)]
pub struct Config { pub namespaces: Namespaces, pub limits: Limits, pub timeouts: Timeouts }
#[derive(Debug, Deserialize, Clone)]
pub struct Namespaces { pub enable: bool, pub pid: bool, pub mount: bool, pub user: bool, pub net: bool }
#[derive(Debug, Deserialize, Clone)]
pub struct Limits { pub cpu_ms: u64, pub mem_mb: u64, pub fsize_mb: u64, pub nproc: u64, pub nofile: u64 }
#[derive(Debug, Deserialize, Clone)]
pub struct Timeouts { pub job_ms: u64, pub grace_ms: u64, pub idle_session_ms: u64 }
impl Config { pub fn load() -> anyhow::Result<Self> { Ok(toml::from_str(&std::fs::read_to_string("config/sandbox.toml")?)?) } }
