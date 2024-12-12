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
    
    pub fn from_solana(solana_address: &str) -> Result<Self, AddressError> {
       
        if solana_address.len() != SOLANA_ADDRESS_LENGTH || 
           !solana_address.chars().all(|c| c.is_ascii_alphanumeric()) {
            return Err(AddressError::InvalidSolanaAddress(
                format!("Invalid Solana address format: must be {} characters long and alphanumeric", 
                       SOLANA_ADDRESS_LENGTH)
            ));
        }

        
        let mut hashes = Vec::new();
        let mut prev_hash = solana_address.to_string();

        
        for i in 0..4 {
            let mut hash: u128 = 5381; // DJB2 初始值
            let input = format!("{}{}", prev_hash, i);

           
            for c in input.chars() {
                hash = ((hash << 5).wrapping_add(hash)).wrapping_add(c as u128);
            }

            
            let hash_hex = format!("{:016x}", hash % (1u128 << 64));
            hashes.push(hash_hex);
            prev_hash = hash_hex;
        }

        
        let dadbs_addr = format!("{}{}", DADBS_PREFIX, hashes.join(""));
        Ok(DADBSAddress(dadbs_addr))
    }

   
    pub fn from_string(dadbs_address: &str) -> Result<Self, AddressError> {
        
        if !dadbs_address.starts_with(DADBS_PREFIX) {
            return Err(AddressError::InvalidDADBSAddress(
                "Invalid DADBS address prefix".to_string()
            ));
        }

       
        let addr_part = &dadbs_address[DADBS_PREFIX.len()..];
        if addr_part.len() != 64 || !addr_part.chars().all(|c| c.is_ascii_hexdigit()) {
            return Err(AddressError::InvalidDADBSAddress(
                format!("Invalid DADBS address format: must be {} characters after prefix and hexadecimal", 64)
            ));
        }

        Ok(DADBSAddress(dadbs_address.to_string()))
    }

    
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
        
        let solana_addr = "DYw8jCTfwHNRJhhmFcbXvVDTqWMEVFBX6ZKUmG5CNSKK";
        
        
        let dadbs_addr = DADBSAddress::from_solana(solana_addr).unwrap();
        
       
        assert!(dadbs_addr.as_string().starts_with(DADBS_PREFIX));
        assert_eq!(dadbs_addr.as_string().len(), DADBS_PREFIX.len() + 64);
    }

    #[test]
    fn test_invalid_solana_address() {
        
        let invalid_length = "DYw8jCTfwHNRJhhmFcbXvVDTqWMEVFBX6ZKUmG5CNS";
        assert!(DADBSAddress::from_solana(invalid_length).is_err());

        
        let invalid_chars = "DYw8jCTfwHNRJhhmFcbXvVDTqWMEVFBX6ZKUmG5CNSKK!@";
        assert!(DADBSAddress::from_solana(invalid_chars).is_err());
    }

    #[test]
    fn test_dadbs_address_from_string() {
        
        let dadbs_addr_str = format!("{}{}",
            DADBS_PREFIX,
            "1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef"
        );
        
        
        let result = DADBSAddress::from_string(&dadbs_addr_str);
        assert!(result.is_ok());
        
        
        let invalid_prefix = "xx1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef";
        assert!(DADBSAddress::from_string(invalid_prefix).is_err());

        
        let invalid_length = format!("{}{}",
            DADBS_PREFIX,
            "1234567890abcdef1234567890abcdef"
        );
        assert!(DADBSAddress::from_string(&invalid_length).is_err());

        
        let invalid_chars = format!("{}{}",
            DADBS_PREFIX,
            "1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdeg"
        );
        assert!(DADBSAddress::from_string(&invalid_chars).is_err());
    }

    #[test]
    fn test_hash_consistency() {
        let solana_addr = "DYw8jCTfwHNRJhhmFcbXvVDTqWMEVFBX6ZKUmG5CNSKK";
        let addr1 = DADBSAddress::from_solana(solana_addr).unwrap();
        let addr2 = DADBSAddress::from_solana(solana_addr).unwrap();
        assert_eq!(addr1, addr2);
    }
}
