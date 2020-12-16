use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Serialize, Deserialize, Default, Debug, Clone)]
pub struct ModConfiguration {
    pub creator: String,
    pub identifier: String,
    pub version: String,
    pub display_name: String,
    pub description: String,
    pub license: String,
    #[serde(default)]
    pub website_url: Option<String>,
    #[serde(default)]
    pub dependancies: Vec<String>,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default)]
    pub install_strategies: Vec<HashMap<String, String>>, //TODO: allow arbitrary content for the value
    #[serde(default)]
    pub extra_data: HashMap<String, String>, //TODO: allow arbitrary content for the value
}
