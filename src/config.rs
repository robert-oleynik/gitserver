use std::path::Path;

use anyhow::{Context, Result};
use tf_bindgen::serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize)]
#[serde(crate = "::tf_bindgen::serde")]
pub struct Config {
    pub server: Server,
    pub root: Root,
}

#[derive(Deserialize, Serialize)]
#[serde(crate = "::tf_bindgen::serde")]
pub struct Server {
    pub node: String,
    pub domain: String,
}

#[derive(Deserialize, Serialize)]
#[serde(crate = "::tf_bindgen::serde")]
pub struct Root {
    pub user: String,
    pub passwd: String,
    pub email: String,
}

impl Config {
    pub fn from_file(path: impl AsRef<Path>) -> Result<Self> {
        let content = std::fs::read_to_string(path).context("failed to read config file")?;
        let config = toml::from_str(&content).context("failed to parse config file")?;
        Ok(config)
    }
}
