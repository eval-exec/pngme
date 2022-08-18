#![allow(unused_variables)]
#![allow(unused_imports)]

use std::fmt::{Debug, Display, Formatter};
use std::str::FromStr;
use anyhow::anyhow;
use crate::{Error, Result};
use crate::chunk_type::ChunkType;


#[derive(Clone)]
pub struct Chunk {
    length: u32,
    chunk_type: ChunkType,
    chunk_data: Vec<u8>,
    crc: u32,

}

#[derive(Debug)]
enum ChunkErr {
    ParseErr,
    CrcVerify,
}


impl Display for ChunkErr {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match *self {
            ChunkErr::ParseErr => write!(f, "parse err"),
            ChunkErr::CrcVerify => write!(f, "verify crc"),
        }
    }
}

impl std::error::Error for ChunkErr {}

impl TryFrom<&Vec<u8>> for Chunk {
    type Error = Error;

    fn try_from(value: &Vec<u8>) -> Result<Self> {
        if value.len() < 12 {
            return Err(Error::from("value length less than 12"));
        }


        let result1: [u8; 4] = value[0..4].to_vec().try_into().unwrap();
        let length = u32::from_be_bytes(result1);
        let length_all = length + 12;

        if length_all != value.len() as u32 {
            return Err(Box::new(ChunkErr::ParseErr));
        }


        let chunk_type_str = String::from_utf8(value[4..8].to_vec())
            .map_err(|_| ("chunk_type is invalid"))?;
        let chunk_type = ChunkType::from_str(&chunk_type_str)?;
        let message: Vec<u8> = value[8..8 + length as usize].to_vec();
        let in_crc_checksum = u32::from_be_bytes(value[length_all as usize - 4..length_all as usize].to_vec().try_into().unwrap());

        let checksum = crc_checksum(&chunk_type, &message);
        if checksum != in_crc_checksum {
            return Err(Box::from(ChunkErr::CrcVerify));
        }
        let chunk = Chunk {
            length,
            chunk_type,
            chunk_data: message,
            crc: in_crc_checksum,
        };


        Ok(chunk)
    }
}

fn crc_checksum(chunk_type: &ChunkType, chunk_data: &Vec<u8>) -> u32 {
    // let data = chunk_type.bytes().iter().chain(chunk_data.as_slice().iter()).copied().collect::<Vec<u8>>();
    let mut data = chunk_type.bytes().to_vec();
    data.append(&mut chunk_data.to_vec());
    crc::crc32::checksum_ieee(&data)
}

impl Display for Chunk {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "length: {},type {}, message:{}, crc: {}", self.length,
               self.chunk_type,
               String::from_utf8_lossy(&self.chunk_data),
               self.crc)
    }
}

impl Chunk {
    pub fn new(chunk_type: ChunkType, data: Vec<u8>) -> Chunk {

        // catencate the chunk type and data together
        let crc_result = crc_checksum(&chunk_type, &data);
        Chunk {
            length: data.len() as u32,
            chunk_type,
            chunk_data: data,
            crc: crc_result,
        }
    }
    pub fn length(&self) -> u32 {
        self.length
    }
    pub fn chunk_type(&self) -> &ChunkType {
        &self.chunk_type
    }
    fn data(&self) -> &[u8] {
        &self.chunk_data
    }
    fn crc(&self) -> u32 {
        self.crc
    }
    pub fn data_as_string(&self) -> Result<String> {
        if let Ok(v) = String::from_utf8(self.chunk_data.to_vec()) {
            Ok(v)
        } else {
            Err(Error::from("data is not valid UTF-8"))
        }
    }
    pub fn as_bytes(&self) -> Vec<u8> {
        self.length
            .to_be_bytes()
            .iter()
            .chain(self.chunk_type().bytes().iter())
            .chain(self.chunk_data.iter())
            .chain(self.crc.to_be_bytes().iter())
            .copied()
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use std::{assert_eq, format};
    use super::*;
    use crate::chunk_type::ChunkType;
    use std::str::FromStr;

    fn testing_chunk() -> Chunk {
        let data_length: u32 = 42;
        let chunk_type = "RuSt".as_bytes();
        let message_bytes = "This is where your secret message will be!".as_bytes();
        let crc: u32 = 2882656334;

        let chunk_data: Vec<u8> = data_length
            .to_be_bytes()
            .iter()
            .chain(chunk_type.iter())
            .chain(message_bytes.iter())
            .chain(crc.to_be_bytes().iter())
            .copied()
            .collect();

        Chunk::try_from(chunk_data.as_ref()).unwrap()
    }

    #[test]
    fn test_new_chunk() {
        let chunk_type = ChunkType::from_str("RuSt").unwrap();
        let data = "This is where your secret message will be!".as_bytes().to_vec();
        let chunk = Chunk::new(chunk_type, data);
        assert_eq!(chunk.length(), 42);
        assert_eq!(chunk.crc(), 2882656334);
    }

    #[test]
    fn test_chunk_length() {
        let chunk = testing_chunk();
        assert_eq!(chunk.length(), 42);
    }

    #[test]
    fn test_chunk_type() {
        let chunk = testing_chunk();
        assert_eq!(chunk.chunk_type().to_string(), String::from("RuSt"));
    }

    #[test]
    fn test_chunk_string() {
        let chunk = testing_chunk();
        let chunk_string = chunk.data_as_string().unwrap();
        let expected_chunk_string = String::from("This is where your secret message will be!");
        assert_eq!(chunk_string, expected_chunk_string);
    }

    #[test]
    fn test_chunk_crc() {
        let chunk = testing_chunk();
        assert_eq!(chunk.crc(), 2882656334);
    }

    #[test]
    fn test_valid_chunk_from_bytes() {
        let data_length: u32 = 42;
        let chunk_type = "RuSt".as_bytes();
        let message_bytes = "This is where your secret message will be!".as_bytes();
        let crc: u32 = 2882656334;

        let chunk_data: Vec<u8> = data_length
            .to_be_bytes()
            .iter()
            .chain(chunk_type.iter())
            .chain(message_bytes.iter())
            .chain(crc.to_be_bytes().iter())
            .copied()
            .collect();

        let chunk = Chunk::try_from(chunk_data.as_ref()).unwrap();

        let chunk_string = chunk.data_as_string().unwrap();
        let expected_chunk_string = String::from("This is where your secret message will be!");

        assert_eq!(chunk.length(), 42);
        assert_eq!(chunk.chunk_type().to_string(), String::from("RuSt"));
        assert_eq!(chunk_string, expected_chunk_string);
        assert_eq!(chunk.crc(), 2882656334);
    }

    #[test]
    fn test_invalid_chunk_from_bytes() {
        let data_length: u32 = 42;
        let chunk_type = "RuSt".as_bytes();
        let message_bytes = "This is where your secret message will be!".as_bytes();
        let crc: u32 = 2882656333;

        let chunk_data: Vec<u8> = data_length
            .to_be_bytes()
            .iter()
            .chain(chunk_type.iter())
            .chain(message_bytes.iter())
            .chain(crc.to_be_bytes().iter())
            .copied()
            .collect();

        let chunk = Chunk::try_from(chunk_data.as_ref());

        assert!(chunk.is_err());
    }

    #[test]
    pub fn test_chunk_trait_impls() {
        let data_length: u32 = 42;
        let chunk_type = "RuSt".as_bytes();
        let message_bytes = "This is where your secret message will be!".as_bytes();
        let crc: u32 = 2882656334;

        let chunk_data: Vec<u8> = data_length
            .to_be_bytes()
            .iter()
            .chain(chunk_type.iter())
            .chain(message_bytes.iter())
            .chain(crc.to_be_bytes().iter())
            .copied()
            .collect();

        let chunk: Chunk = TryFrom::try_from(chunk_data.as_ref()).unwrap();

        let _chunk_string = format!("{}", chunk);
    }
}
