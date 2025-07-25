use anyhow::Result;
use config_derive::Merge;
use serde::Deserialize;
use std::{
    env::{self},
    fs::File,
    io::Read,
    path::Path,
};

#[derive(Deserialize, Merge, Clone, Debug, Default)]
pub(crate) struct Config {
    pub(crate) user: Option<User>,
    pub(crate) init: Option<Init>,
    pub(crate) commit: Option<Commit>,
}

#[derive(Deserialize, Merge, Clone, Debug, Default)]
pub(crate) struct User {
    pub(crate) name: Option<String>,
    pub(crate) email: Option<String>,
}

#[derive(Deserialize, Merge, Clone, Debug)]
pub(crate) struct Init {
    pub(crate) default_branch: Option<String>,
}

#[derive(Deserialize, Merge, Clone, Debug)]
pub(crate) struct Commit {
    /// the command to start the editor for commit messages.
    /// the filename is given as second argument
    pub(crate) editor: Option<String>,
}

impl Default for Init {
    fn default() -> Self {
        Self {
            default_branch: Some("master".to_string()),
        }
    }
}

pub(crate) const CONFIG_PATHS: [&str; 6] = [
    "/etc/dcgconfig.toml",
    "$XDG_CONFIG_HOME$/dcg/config.toml",
    "$HOME$/.config/dcg/config.toml",
    "$HOME$/.dcgconfig.toml",
    "$PWD$/.dcg/config.toml",
    "$PWD$/.dcgconfig.toml",
];

fn replace_variables(s: &str) -> Result<String> {
    let mut out = String::new();

    for (i, x) in s.split('$').enumerate() {
        if i % 2 == 0 {
            out.push_str(x);
        } else {
            match env::var(x) {
                Ok(x) => out.push_str(&x),
                _ if x == "PWD" => {
                    out.push_str(env::current_dir()?.as_os_str().to_str().unwrap_or("."))
                }
                _ => {}
            }
        }
    }

    Ok(out)
}

fn extract_config(p: &Path, cfg: Config) -> Result<Config> {
    let mut s = String::new();

    File::open(p)?.read_to_string(&mut s)?;

    let new: Config = toml::from_str(&s)?;

    Ok(cfg.merge(new))
}

pub(crate) fn read_config() -> Result<Config> {
    let mut config = Config::default();

    for path in CONFIG_PATHS {
        let path = replace_variables(path)?;
        let path = Path::new(&path);

        if path.exists() {
            config = extract_config(path, config)?;
        }
    }

    Ok(config)
}
