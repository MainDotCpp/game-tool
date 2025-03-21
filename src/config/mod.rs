use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use std::env;

pub mod backup;

#[derive(Debug, Serialize, Deserialize)]
pub struct Account {
    pub username: String,
    pub password: String,
    pub note: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncGroup {
    pub name: String,
    pub description: Option<String>,
    pub is_enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncItem {
    pub name: String,
    pub source_path: PathBuf,
    pub backup_path: PathBuf,
    pub is_enabled: bool,
    pub group: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    pub accounts: Vec<Account>,
    pub game_config_path: PathBuf,
    pub script_paths: Vec<PathBuf>,
    pub backup_dir: PathBuf,
    pub sync_items: Vec<SyncItem>,
    pub sync_groups: Vec<SyncGroup>,
}

impl Default for Config {
    fn default() -> Self {
        let default_backup_dir = get_exe_dir().join("backups");
        
        Config {
            accounts: Vec::new(),
            game_config_path: PathBuf::from(r"C:\Riot Games\League of Legends\Config"),
            script_paths: Vec::new(),
            backup_dir: default_backup_dir,
            sync_items: Vec::new(),
            sync_groups: Vec::new(),
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
            .with_context(|| format!("读取配置文件失败: {:?}", config_path))?;
            
        let config: Config = serde_json::from_str(&config_str)
            .with_context(|| "解析配置文件失败")?;
            
        Ok(config)
    }
    
    pub fn save(&self) -> Result<()> {
        let config_path = get_config_path();
        
        if let Some(parent) = config_path.parent() {
            fs::create_dir_all(parent)
                .with_context(|| format!("创建配置目录失败: {:?}", parent))?;
        }
        
        let config_str = serde_json::to_string_pretty(self)
            .with_context(|| "序列化配置失败")?;
            
        fs::write(&config_path, config_str)
            .with_context(|| format!("写入配置文件失败: {:?}", config_path))?;
            
        Ok(())
    }
    
    pub fn ensure_backup_dir(&self) -> Result<()> {
        fs::create_dir_all(&self.backup_dir)
            .with_context(|| format!("创建备份目录失败: {:?}", self.backup_dir))?;
        Ok(())
    }
}

fn get_exe_dir() -> PathBuf {
    env::current_exe()
        .map(|path| path.parent().unwrap_or(std::path::Path::new(".")).to_path_buf())
        .unwrap_or_else(|_| PathBuf::from("."))
}

pub fn get_config_path() -> PathBuf {
    get_exe_dir().join("config.json")
} 