use serde::{Deserialize, Serialize};
use std::collections::HashMap;
#[derive(Serialize, Deserialize, Debug)]
pub struct Config {
    pub version: u16,
    #[serde(alias = "d")]
    pub dotfiles: HashMap<String, ConfigItem>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ConfigItem {
    pub(crate) mappings: Vec<String>,
    pub(crate) before: Option<Vec<String>>,
    pub(crate) after: Option<Vec<String>>,
}
