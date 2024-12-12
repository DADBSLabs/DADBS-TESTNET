use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;
use std::net::ToSocketAddrs;
use log::{warn, error};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ConfigError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("TOML parsing error: {0}")]
    Toml(#[from] toml::de::Error),
    #[error("Invalid host or port: {0}")]
    InvalidAddress(String),
    #[error("Invalid bootstrap node address: {0}")]
    InvalidBootstrapNode(String),
    #[error("Storage path error: {0}")]
    StoragePath(String),
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct NodeConfig {
    pub node_id: String,
    pub host: String,
    pub port: u16,
    pub storage_path: String,
    pub max_connections: u32,
    pub consensus_timeout: u64,   // 共识超时时间（毫秒）
    pub bootstrap_nodes: Vec<String>, // 引导节点列表
    #[serde(default)]
    pub llm: Option<LLMConfig>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct LLMConfig {
    pub enabled: bool,
    pub model_path: String,
    pub tokenizer_path: String,
    pub max_batch_size: usize,
    pub use_gpu: bool,
}

impl Default for NodeConfig {
    fn default() -> Self {
        NodeConfig {
            node_id: uuid::Uuid::new_v4().to_string(),
            host: "127.0.0.1".to_string(),
            port: 8000,
            storage_path: "./data".to_string(),
            max_connections: 50,
            consensus_timeout: 5000,
            bootstrap_nodes: vec![
                "testnet.dadbs.io:8000".to_string(),
                "testnet2.dadbs.io:8000".to_string(),
            ],
            llm: None,
        }
    }
}

impl NodeConfig {
    pub fn load(path: &Path) -> Result<Self, ConfigError> {
        let config_str = fs::read_to_string(path)?;
        let mut config: NodeConfig = toml::from_str(&config_str)?;
        
        // 验证配置
        config.validate()?;
        
        // 创建存储目录
        let storage_path = Path::new(&config.storage_path);
        if !storage_path.exists() {
            fs::create_dir_all(storage_path).map_err(|e| {
                ConfigError::StoragePath(format!("Failed to create storage directory: {}", e))
            })?;
        }

        // 验证 LLM 配置（如果启用）
        if let Some(llm_config) = &config.llm {
            if llm_config.enabled {
                // 检查模型文件
                let model_path = Path::new(&llm_config.model_path);
                if !model_path.exists() {
                    return Err(ConfigError::StoragePath(
                        format!("LLM model file not found: {}", model_path.display())
                    ));
                }
                
                // 检查分词器文件
                let tokenizer_path = Path::new(&llm_config.tokenizer_path);
                if !tokenizer_path.exists() {
                    return Err(ConfigError::StoragePath(
                        format!("LLM tokenizer file not found: {}", tokenizer_path.display())
                    ));
                }
            }
        }

        Ok(config)
    }

    pub fn save(&self, path: &Path) -> Result<(), ConfigError> {
        // 验证配置
        self.validate()?;
        
        let config_str = toml::to_string_pretty(self)
            .map_err(ConfigError::Toml)?;
        
        // 确保父目录存在
        if let Some(parent) = path.parent() {
            if !parent.exists() {
                fs::create_dir_all(parent)?;
            }
        }
        
        fs::write(path, config_str)?;
        Ok(())
    }

    fn validate(&self) -> Result<(), ConfigError> {
        // 验证监听地址
        let addr = format!("{}:{}", self.host, self.port);
        addr.to_socket_addrs()
            .map_err(|_| ConfigError::InvalidAddress(addr.clone()))?;

        // 验证引导节点地址
        for node in &self.bootstrap_nodes {
            node.to_socket_addrs()
                .map_err(|_| ConfigError::InvalidBootstrapNode(node.clone()))?;
        }

        // 验证端口范围
        if self.port < 1024 && self.port != 0 {
            warn!("Using privileged port {}, this might require root/admin privileges", self.port);
        }

        // 验证连接数限制
        if self.max_connections > 1000 {
            warn!("High max_connections value ({}), this might consume significant resources", self.max_connections);
        }

        // 验证超时设置
        if self.consensus_timeout < 1000 {
            warn!("Very low consensus_timeout ({}ms), this might cause consensus issues", self.consensus_timeout);
        }

        // 验证存储路径
        let storage_path = Path::new(&self.storage_path);
        if storage_path.exists() && !storage_path.is_dir() {
            return Err(ConfigError::StoragePath(
                "Storage path exists but is not a directory".to_string()
            ));
        }

        Ok(())
    }
}
