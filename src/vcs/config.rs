use anyhow::Result;
use clap::builder::OsStr;
use config_derive::Merge;
use serde::Deserialize;
use std::{
    env::{self, VarError},
    fs::File,
    io::{self, Read},
    path::Path,
};

#[derive(Deserialize, Clone, Debug, Default)]
pub(crate) struct Config {
    pub(crate) user: User,
    pub(crate) init: Init,
}

impl Config {
    fn merge(self, other: Self) -> Self {
        Self {
            user: self.user.merge(other.user),
            init: self.init.merge(other.init),
        }
    }
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

impl Default for Init {
    fn default() -> Self {
        Self {
            default_branch: Some("master".to_string()),
        }
    }
}

pub(crate) const CONFIG_PATHS: [&str; 6] = [
    "/etc/dcgconfig",
    "$XDG_CONFIG_HOME$/dcg/config",
    "$HOME$/.config/dcg/config",
    "$HOME$/.dcgconfig",
    "$PWD$/.dcg/config",
    "$PWD$/.dcgconfig",
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

fn extract_config(p: &Path, cfg: &mut Config) -> Result<()> {
    let mut s = String::new();

    File::open(p)?.read_to_string(&mut s)?;

    let new: Config = toml::from_str(&s)?;

    Ok(())
}

pub(crate) fn read_config() -> Result<Config> {
    let mut config = Config::default();

    for path in CONFIG_PATHS {
        let path = replace_variables(path)?;
        let path = Path::new(&path);

        if path.exists() {
            extract_config(path, &mut config)?;
        }
    }

    Ok(config)
}
