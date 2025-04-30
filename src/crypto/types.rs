use std::fmt;
use std::str::FromStr;
use failure::{Fail, format_err};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EncryptionType {
    ECDSA,
    FNDSA,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DecryptionType {
    ECDSA,
    FNDSA,
}

#[derive(Debug, Fail)]
pub enum CryptoTypeError {
    #[fail(display = "Fail Encrypt type: {}", _0)]
    InvalidEncryptionTypes(String),

    #[fail(display = "Fail Decrypt type: {}", _0)]
    InvalidDecryptionType(String),
}

impl fmt::Display for EncryptionType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            EncryptionType::ECDSA => write!(f, "ECDSA"),
            EncryptionType::FNDSA => write!(f, "FNDSA"),
        }
    }
}

impl fmt::Display for DecryptionType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DecryptionType::ECDSA => write!(f, "ECDSA"),
            DecryptionType::FNDSA => write!(f, "FNDSA"),
        }
    }
}

impl FromStr for EncryptionType {
    type Err = CryptoTypeError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_uppercase().as_str() {
            "ECDSA" => Ok(EncryptionType::ECDSA),
            "FNDSA" => Ok(EncryptionType::FNDSA),
            _ => Err(CryptoTypeError::InvalidEncryptionTypes(s.to_string())),
        }
    }
}

impl FromStr for DecryptionType {
    type Err = CryptoTypeError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_uppercase().as_str() {
            "ECDSA" => Ok(DecryptionType::ECDSA),
            "FNDSA" => Ok(DecryptionType::FNDSA),
            _ => Err(CryptoTypeError::InvalidDecryptionType(s.to_string())),
        }
    }
}

impl EncryptionType {
    pub fn as_str(&self) -> &'static str {
        match self {
            EncryptionType::ECDSA => "ECDSA",
            EncryptionType::FNDSA => "FNDSA",
        }
    }
    
    pub fn to_decryption_type(&self) -> DecryptionType {
        match self {
            EncryptionType::ECDSA => DecryptionType::ECDSA,
            EncryptionType::FNDSA => DecryptionType::FNDSA,
        }
    }
}

impl DecryptionType {
    pub fn as_str(&self) -> &'static str {
        match self {
            DecryptionType::ECDSA => "ECDSA",
            DecryptionType::FNDSA => "FNDSA",
        }
    }

    pub fn to_encryption_type(&self) -> EncryptionType {
        match self {
            DecryptionType::ECDSA => EncryptionType::ECDSA,
            DecryptionType::FNDSA => EncryptionType::FNDSA,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encryption_type_from_str() {
        assert_eq!(EncryptionType::from_str("ECDSA").unwrap(), EncryptionType::ECDSA);
        assert_eq!(EncryptionType::from_str("ecdsa").unwrap(), EncryptionType::ECDSA);
        assert_eq!(EncryptionType::from_str("FNDSA").unwrap(), EncryptionType::FNDSA);
        assert_eq!(EncryptionType::from_str("fndsa").unwrap(), EncryptionType::FNDSA);
        assert!(EncryptionType::from_str("invalid").is_err());
    }

    #[test]
    fn test_decryption_type_from_str() {
        assert_eq!(DecryptionType::from_str("ECDSA").unwrap(), DecryptionType::ECDSA);
        assert_eq!(DecryptionType::from_str("ecdsa").unwrap(), DecryptionType::ECDSA);
        assert_eq!(DecryptionType::from_str("FNDSA").unwrap(), DecryptionType::FNDSA);
        assert_eq!(DecryptionType::from_str("fndsa").unwrap(), DecryptionType::FNDSA);
        assert!(DecryptionType::from_str("invalid").is_err());
    }

    #[test]
    fn test_conversion_between_types() {
        assert_eq!(EncryptionType::ECDSA.to_decryption_type(), DecryptionType::ECDSA);
        assert_eq!(EncryptionType::FNDSA.to_decryption_type(), DecryptionType::FNDSA);
        assert_eq!(DecryptionType::ECDSA.to_encryption_type(), EncryptionType::ECDSA);
        assert_eq!(DecryptionType::FNDSA.to_encryption_type(), EncryptionType::FNDSA);
    }

    #[test]
    fn test_display_trait() {
        assert_eq!(EncryptionType::ECDSA.to_string(), "ECDSA");
        assert_eq!(EncryptionType::FNDSA.to_string(), "FNDSA");
        assert_eq!(DecryptionType::ECDSA.to_string(), "ECDSA");
        assert_eq!(DecryptionType::FNDSA.to_string(), "FNDSA");
    }
}
