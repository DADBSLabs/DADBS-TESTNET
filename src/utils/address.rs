use std::fmt;
use thiserror::Error;

const DADBS_PREFIX: &str = "dadbs";
const SOLANA_ADDRESS_LENGTH: usize = 44;

#[derive(Error, Debug)]
pub enum AddressError {
    #[error("Invalid Solana address: {0}")]
    InvalidSolanaAddress(String),
    #[error("Invalid DADBS address: {0}")]
    InvalidDADBSAddress(String),
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct DADBSAddress(String);

impl DADBSAddress {
    /// 从 Solana 地址创建 DADBS 地址
    pub fn from_solana(solana_address: &str) -> Result<Self, AddressError> {
        // 验证 Solana 地址格式
        if solana_address.len() != SOLANA_ADDRESS_LENGTH || 
           !solana_address.chars().all(|c| c.is_ascii_alphanumeric()) {
            return Err(AddressError::InvalidSolanaAddress(
                format!("Invalid Solana address format: must be {} characters long and alphanumeric", 
                       SOLANA_ADDRESS_LENGTH)
            ));
        }

        // 使用 DJB2 哈希算法生成多个哈希
        let mut hashes = Vec::new();
        let mut prev_hash = solana_address.to_string();

        // 生成 4 个不同的哈希
        for i in 0..4 {
            let mut hash: u128 = 5381; // DJB2 初始值
            let input = format!("{}{}", prev_hash, i);

            // DJB2 哈希算法
            for c in input.chars() {
                hash = ((hash << 5).wrapping_add(hash)).wrapping_add(c as u128);
            }

            // 转换为 16 位的十六进制字符串
            let hash_hex = format!("{:016x}", hash % (1u128 << 64));
            hashes.push(hash_hex);
            prev_hash = hash_hex;
        }

        // 组合所有哈希并添加前缀
        let dadbs_addr = format!("{}{}", DADBS_PREFIX, hashes.join(""));
        Ok(DADBSAddress(dadbs_addr))
    }

    /// 从 DADBS 地址字符串创建 DADBSAddress
    pub fn from_string(dadbs_address: &str) -> Result<Self, AddressError> {
        // 验证前缀
        if !dadbs_address.starts_with(DADBS_PREFIX) {
            return Err(AddressError::InvalidDADBSAddress(
                "Invalid DADBS address prefix".to_string()
            ));
        }

        // 验证剩余部分
        let addr_part = &dadbs_address[DADBS_PREFIX.len()..];
        if addr_part.len() != 64 || !addr_part.chars().all(|c| c.is_ascii_hexdigit()) {
            return Err(AddressError::InvalidDADBSAddress(
                format!("Invalid DADBS address format: must be {} characters after prefix and hexadecimal", 64)
            ));
        }

        Ok(DADBSAddress(dadbs_address.to_string()))
    }

    /// 获取地址字符串
    pub fn as_string(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for DADBSAddress {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_solana_to_dadbs_conversion() {
        // 示例 Solana 地址
        let solana_addr = "DYw8jCTfwHNRJhhmFcbXvVDTqWMEVFBX6ZKUmG5CNSKK";
        
        // 转换为 DADBS 地址
        let dadbs_addr = DADBSAddress::from_solana(solana_addr).unwrap();
        
        // 验证格式
        assert!(dadbs_addr.as_string().starts_with(DADBS_PREFIX));
        assert_eq!(dadbs_addr.as_string().len(), DADBS_PREFIX.len() + 64);
    }

    #[test]
    fn test_invalid_solana_address() {
        // 测试无效的 Solana 地址（长度错误）
        let invalid_length = "DYw8jCTfwHNRJhhmFcbXvVDTqWMEVFBX6ZKUmG5CNS";
        assert!(DADBSAddress::from_solana(invalid_length).is_err());

        // 测试无效字符
        let invalid_chars = "DYw8jCTfwHNRJhhmFcbXvVDTqWMEVFBX6ZKUmG5CNSKK!@";
        assert!(DADBSAddress::from_solana(invalid_chars).is_err());
    }

    #[test]
    fn test_dadbs_address_from_string() {
        // 创建有效的 DADBS 地址
        let dadbs_addr_str = format!("{}{}",
            DADBS_PREFIX,
            "1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef"
        );
        
        // 测试有效地址
        let result = DADBSAddress::from_string(&dadbs_addr_str);
        assert!(result.is_ok());
        
        // 测试无效前缀
        let invalid_prefix = "xx1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef";
        assert!(DADBSAddress::from_string(invalid_prefix).is_err());

        // 测试无效长度
        let invalid_length = format!("{}{}",
            DADBS_PREFIX,
            "1234567890abcdef1234567890abcdef"
        );
        assert!(DADBSAddress::from_string(&invalid_length).is_err());

        // 测试无效字符
        let invalid_chars = format!("{}{}",
            DADBS_PREFIX,
            "1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdeg"
        );
        assert!(DADBSAddress::from_string(&invalid_chars).is_err());
    }

    #[test]
    fn test_hash_consistency() {
        // 测试相同的输入产生相同的输出
        let solana_addr = "DYw8jCTfwHNRJhhmFcbXvVDTqWMEVFBX6ZKUmG5CNSKK";
        let addr1 = DADBSAddress::from_solana(solana_addr).unwrap();
        let addr2 = DADBSAddress::from_solana(solana_addr).unwrap();
        assert_eq!(addr1, addr2);
    }
}
