use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

pub mod backup;

#[derive(Debug, Serialize, Deserialize)]
pub struct Account {
    pub username: String,
    pub password: String,
    pub note: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    pub accounts: Vec<Account>,
    pub game_config_path: PathBuf,
    pub script_paths: Vec<PathBuf>,
    pub backup_dir: PathBuf,
}

impl Default for Config {
    fn default() -> Self {
        let default_backup_dir = dirs::data_local_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("game_tool_backups");
        
        Config {
            accounts: Vec::new(),
            game_config_path: PathBuf::from(r"C:\Riot Games\League of Legends\Config"),
            script_paths: Vec::new(),
            backup_dir: default_backup_dir,
        }
    }
}

impl Config {
    pub fn load() -> Result<Self> {
        let config_path = get_config_path();
        
        if !config_path.exists() {
            let default_config = Config::default();
            default_config.save()?;
            return Ok(default_config);
        }
        
        let config_str = fs::read_to_string(&config_path)
            .with_context(|| format!("Failed to read config file: {:?}", config_path))?;
            
        let config: Config = serde_json::from_str(&config_str)
            .with_context(|| "Failed to parse config file")?;
            
        Ok(config)
    }
    
    pub fn save(&self) -> Result<()> {
        let config_path = get_config_path();
        
        // Ensure parent directory exists
        if let Some(parent) = config_path.parent() {
            fs::create_dir_all(parent)
                .with_context(|| format!("Failed to create config directory: {:?}", parent))?;
        }
        
        let config_str = serde_json::to_string_pretty(self)
            .with_context(|| "Failed to serialize config")?;
            
        fs::write(&config_path, config_str)
            .with_context(|| format!("Failed to write config file: {:?}", config_path))?;
            
        Ok(())
    }
    
    pub fn ensure_backup_dir(&self) -> Result<()> {
        fs::create_dir_all(&self.backup_dir)
            .with_context(|| format!("Failed to create backup directory: {:?}", self.backup_dir))?;
        Ok(())
    }
}

pub fn get_config_path() -> PathBuf {
    dirs::config_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("game_tool")
        .join("config.json")
} 