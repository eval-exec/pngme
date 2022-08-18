#![allow(unused_imports)]

use std::fmt::Formatter;
use std::io::Read;
use std::str::FromStr;
use crate::{Error, Result};

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct ChunkType([u8; 4]);

impl std::fmt::Display for ChunkType {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match String::from_utf8(self.0.to_vec()) {
            Ok(s) => {
                write!(f, "{}", s)
            }
            Err(e) => {
                write!(f, "{}", e)
            }
        }
    }
}

impl TryFrom<[u8; 4]> for ChunkType {
    type Error = Error;

    fn try_from(value: [u8; 4]) -> Result<ChunkType> {
        Ok(ChunkType(value))
    }
}

impl FromStr for ChunkType {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self> {
        if s.len() != 4 {
            return Err(Error::from("invalid str"));
        }
        for c in s.chars() {
            if !c.is_ascii_uppercase() && !c.is_ascii_lowercase() {
                return Err(Error::from("invalid character"));
            }
        }

        let ck = s.as_bytes().to_vec();
        Ok(ChunkType(ck.try_into().unwrap()))
    }
}


impl ChunkType {
    pub fn bytes(&self) -> [u8; 4] {
        self.0
    }
    pub fn is_valid(&self) -> bool {
        self.is_reserved_bit_valid()
    }
    pub fn is_critical(&self) -> bool {
        self.0[0] & 0b00100000 == 0
    }
    pub fn is_public(&self) -> bool {
        self.0[1] & 0b00100000 == 0
    }
    pub fn is_reserved_bit_valid(&self) -> bool {
        self.0[2] & 0b00100000 == 0
    }
    pub fn is_safe_to_copy(&self) -> bool {
        self.0[3] & 0b00100000 != 0
    }
}

#[cfg(test)]
mod tests {
    use std::{assert_eq, format};
    use super::*;
    use std::convert::TryFrom;
    use std::io::Read;
    use std::str::FromStr;

    #[test]
    pub fn test_chunk_type_from_bytes() {
        let expected = [82, 117, 83, 116];
        let actual = ChunkType::try_from([82, 117, 83, 116]).unwrap();

        assert_eq!(expected, actual.bytes());
    }

    #[test]
    pub fn test_chunk_type_from_str() {
        let expected = ChunkType::try_from([82, 117, 83, 116]).unwrap();
        let actual = ChunkType::from_str("RuSt").unwrap();
        assert_eq!(expected, actual);
    }

    #[test]
    pub fn test_chunk_type_is_critical() {
        let chunk = ChunkType::from_str("RuSt").unwrap();
        assert!(chunk.is_critical());
    }

    #[test]
    pub fn test_chunk_type_is_not_critical() {
        let chunk = ChunkType::from_str("ruSt").unwrap();
        assert!(!chunk.is_critical());
    }

    #[test]
    pub fn test_chunk_type_is_public() {
        let chunk = ChunkType::from_str("RUSt").unwrap();
        assert!(chunk.is_public());
    }

    #[test]
    pub fn test_chunk_type_is_not_public() {
        let chunk = ChunkType::from_str("RuSt").unwrap();
        assert!(!chunk.is_public());
    }

    #[test]
    pub fn test_chunk_type_is_reserved_bit_valid() {
        let chunk = ChunkType::from_str("RuSt").unwrap();
        assert!(chunk.is_reserved_bit_valid());
    }

    #[test]
    pub fn test_chunk_type_is_reserved_bit_invalid() {
        let chunk = ChunkType::from_str("Rust").unwrap();
        assert!(!chunk.is_reserved_bit_valid());
    }

    #[test]
    pub fn test_chunk_type_is_safe_to_copy() {
        let chunk = ChunkType::from_str("RuSt").unwrap();
        assert!(chunk.is_safe_to_copy());
    }

    #[test]
    pub fn test_chunk_type_is_unsafe_to_copy() {
        let chunk = ChunkType::from_str("RuST").unwrap();
        assert!(!chunk.is_safe_to_copy());
    }

    #[test]
    pub fn test_valid_chunk_is_valid() {
        let chunk = ChunkType::from_str("RuSt").unwrap();
        assert!(chunk.is_valid());
    }

    #[test]
    pub fn test_invalid_chunk_is_valid() {
        let chunk = ChunkType::from_str("Rust").unwrap();
        assert!(!chunk.is_valid());

        let chunk = ChunkType::from_str("Ru1t");
        assert!(chunk.is_err());
    }

    #[test]
    pub fn test_chunk_type_string() {
        let chunk = ChunkType::from_str("RuSt").unwrap();
        assert_eq!(&chunk.to_string(), "RuSt");
    }

    #[test]
    pub fn test_chunk_type_trait_impls() {
        let chunk_type_1: ChunkType = TryFrom::try_from([82, 117, 83, 116]).unwrap();
        let chunk_type_2: ChunkType = FromStr::from_str("RuSt").unwrap();
        let _chunk_string = format!("{}", chunk_type_1);
        let _are_chunks_equal = chunk_type_1 == chunk_type_2;
    }
}

