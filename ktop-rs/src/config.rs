use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Serialize, Deserialize, Default)]
pub struct Config {
    #[serde(default)]
    pub theme: Option<String>,
    #[serde(default)]
    pub color_mode: Option<ColorMode>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ColorMode {
    Truecolor,
    Basic,
}

impl Default for ColorMode {
    fn default() -> Self {
        Self::Truecolor
    }
}

impl ColorMode {
    pub fn toggle(self) -> Self {
        match self {
            Self::Truecolor => Self::Basic,
            Self::Basic => Self::Truecolor,
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::Truecolor => "Truecolor",
            Self::Basic => "Basic",
        }
    }
}

fn config_path() -> PathBuf {
    dirs::config_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("ktop")
        .join("config.json")
}

pub fn load_config() -> Config {
    let path = config_path();
    if let Ok(data) = fs::read_to_string(&path) {
        serde_json::from_str(&data).unwrap_or_default()
    } else {
        Config::default()
    }
}

pub fn save_config(cfg: &Config) {
    let path = config_path();
    if let Some(parent) = path.parent() {
        let _ = fs::create_dir_all(parent);
    }
    if let Ok(data) = serde_json::to_string_pretty(cfg) {
        let _ = fs::write(&path, data + "\n");
    }
}
