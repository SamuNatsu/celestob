use std::{collections::HashSet, fs, sync::OnceLock};

use anyhow::{bail, Result};
use serde::Deserialize;

#[derive(Deserialize)]
pub struct Config {
    #[serde(default = "Config::default_bind_addr")]
    pub bind_addr: String,

    #[serde(default = "Config::default_bind_port")]
    pub bind_port: u16,

    pub db_url: String,

    #[serde(default)]
    pub services: Vec<ConfigService>,
}

impl Config {
    pub fn get_instance() -> &'static Self {
        static I: OnceLock<Config> = OnceLock::new();
        I.get_or_init(|| {
            let contents = fs::read_to_string("config.toml").expect("fail to read config file");
            let value = toml::from_str::<Self>(&contents).expect("fail to parse config file");
            value.verify().expect("fail to verify config file");

            value
        })
    }

    fn verify(&self) -> Result<()> {
        let mut set = HashSet::new();
        for s in &self.services {
            if let ConfigService::Http { token, .. } = s {
                if !set.insert(token.clone()) {
                    bail!("token `{}` duplicated", token);
                }
            }
        }

        set.clear();
        for s in &self.services {
            if let ConfigService::Docker { container, .. } = s {
                if !set.insert(container.clone()) {
                    bail!("container `{}` duplicated", container);
                }
            }
        }

        Ok(())
    }

    fn default_bind_addr() -> String {
        "0.0.0.0".into()
    }
    fn default_bind_port() -> u16 {
        3000
    }
}

#[derive(Deserialize)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum ConfigService {
    Http {
        name: String,
        description: String,
        token: String,
    },
    Docker {
        name: String,
        description: String,
        container: String,
    },
}

impl ConfigService {
    pub fn get_name(&self) -> &String {
        match self {
            Self::Http { name, .. } => name,
            Self::Docker { name, .. } => name,
        }
    }
}
