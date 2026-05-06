use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use anyhow::Result;
use log::{info, warn};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub authorized: bool,
    pub auto_start: bool,
    pub port: u16,
    pub version: String,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            authorized: false,
            auto_start: true,
            port: 18888,
            version: "2.1.0".to_string(),
        }
    }
}

impl AppConfig {
    pub fn load() -> Self {
        match Self::load_from_file() {
            Ok(config) => config,
            Err(e) => {
                warn!("加载配置文件失败，使用默认配置: {}", e);
                Self::default()
            }
        }
    }

    fn load_from_file() -> Result<Self> {
        let config_path = Self::get_config_path()?;
        if !config_path.exists() {
            info!("配置文件不存在，创建默认配置");
            let default_config = Self::default();
            default_config.save()?;
            return Ok(default_config);
        }

        let config_str = fs::read_to_string(&config_path)?;
        let config: Self = serde_json::from_str(&config_str)?;
        info!("配置文件加载成功: {:?}", config_path);
        Ok(config)
    }

    pub fn save(&self) -> Result<()> {
        let config_path = Self::get_config_path()?;
        
        // 确保配置目录存在
        if let Some(parent) = config_path.parent() {
            fs::create_dir_all(parent)?;
        }

        let config_str = serde_json::to_string_pretty(self)?;
        fs::write(&config_path, config_str)?;
        info!("配置文件保存成功: {:?}", config_path);
        Ok(())
    }

    fn get_config_path() -> Result<PathBuf> {
        let config_dir = dirs::config_dir()
            .ok_or_else(|| anyhow::anyhow!("无法获取配置目录"))?
            .join("machine-code-tool");
        
        Ok(config_dir.join("config.json"))
    }

    pub fn get_data_dir() -> Result<PathBuf> {
        let data_dir = dirs::data_dir()
            .ok_or_else(|| anyhow::anyhow!("无法获取数据目录"))?
            .join("machine-code-tool");
        
        fs::create_dir_all(&data_dir)?;
        Ok(data_dir)
    }
}