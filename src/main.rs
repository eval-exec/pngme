extern crate core;

use std::str::FromStr;
use clap::{arg, ArgAction, command, Parser, value_parser};
use clap::error::ContextValue::String;
use crate::chunk::Chunk;
use crate::chunk_type::ChunkType;
use crate::png::Png;

mod args;
mod chunk;
mod chunk_type;
mod commands;
mod png;

pub type Error = Box<dyn std::error::Error>;
pub type Result<T> = std::result::Result<T, Error>;

/*

pngme encode ./dice.png ruSt "This is a secret message!

pngme decode ./dice.png ruSt

pngme remove ./dice.png ruSt

pngme print ./dice.png

 */

fn main() -> Result<()> {
    let matches = command!() // requires `cargo` feature
        .arg(arg!([action] "action"))
        .arg(
            arg!([path] "path")
        )
        .arg(
            arg!([chunk_type] "chunk_type").default_missing_value("")
        )
        .arg(
            arg!([message] "message").default_missing_value("")
        )
        .get_matches();

    let mut action;
    if let Some(v) = matches.get_one::<std::string::String>("action") {
        action = (*v).clone()
    } else {
        panic!("need an action")
    }

    let mut path;
    if let Some(v) = matches.get_one::<std::string::String>("path") {
        path = (*v).clone()
    } else {
        panic!("need an path")
    }

    let mut chunk_type = std::string::String::from("");
    if let Some(v) = matches.get_one::<std::string::String>("chunk_type") {
        chunk_type = (*v).clone();
    }
    let mut message: std::string::String = std::string::String::from("");
    if let Some(v) = matches.get_one::<std::string::String>("message") {
        message = (*v).clone();
    }

    println!("{} {} {} {}",action, path, chunk_type, message);
    let png_vec= std::fs::read(&path).unwrap();
    let png_bytes = png_vec.as_slice();
    let mut png = Png::try_from(png_bytes).unwrap();
    match action.as_str() {
        "encode" => {
            let chunk = Chunk::new(ChunkType::from_str(&chunk_type)?, message.as_bytes().to_vec());
            png.append_chunk(chunk);
            std::fs::write(path, png.as_bytes()).expect("write failed")
        }
        "decode" => {
            if let Some(chunk) = png.chunk_by_type(&chunk_type) {
                println!("{}", chunk);
            } else {
                println!("no chunk found")
            }
        }
        "remove" => {
            while let Some(_) = png.chunk_by_type(&chunk_type) {
                png.remove_chunk(&chunk_type).expect("remove chunk failed");
            }
            std::fs::write(path, png.as_bytes()).expect("write failed")
        }
        "print" => {
            png.chunks().iter().for_each(|v| {
                println!("{}", v)
            });
        }
        _ => {
            panic!("unknown action")
        }
    }

    return Ok(());
}

