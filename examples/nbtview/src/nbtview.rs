
pub mod nbt {
    protospec::include_spec!("nbt");
}
use nbt::*;

use std::{fmt, fmt::Write};

use clap::Parser;
use indenter::indented;

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    input: String,
}

#[cfg(not(feature = "async"))]
fn main() {
    let args = Args::parse();
    let file = std::fs::File::open(&*args.input).expect("failed to open file");
    let mut file_reader = std::io::BufReader::new(file);
    let nbt = nbt::Compound::decode_sync(&mut file_reader).expect("failed to decode nbt");
    println!("{}", nbt);
}

#[cfg(feature = "async")]
#[tokio::main]
async fn main() {
    let args = Args::parse();
    let file = tokio::fs::File::open(&*args.input).await.expect("failed to open file");
    let mut file_reader = tokio::io::BufReader::new(file);
    let nbt = nbt::Compound::decode_async(&mut file_reader).await.expect("failed to decode nbt");
    println!("{}", nbt);
}

impl fmt::Display for Compound {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for item in &self.items {
            write!(indented(f).with_str("  "), "\n{}<{}> = {}", item.type_id, item.name, item.payload)?;
        }
        Ok(())
    }
}

impl fmt::Display for List {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}[{}]", self.type_id, self.length)?;
        for item in &self.items {
            write!(indented(f).with_str("  "), "\n{},", item)?;
        }
        Ok(())
    }
}

impl fmt::Display for IntArray {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Int[{}]", self.len)?;
        for item in &self.value {
            write!(indented(f).with_str("  "), "\n{},", item)?;
        }
        Ok(())
    }
}

impl fmt::Display for LongArray {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Long[{}]", self.len)?;
        for item in &self.value {
            write!(indented(f).with_str("  "), "\n{},", item)?;
        }
        Ok(())
    }
}

impl fmt::Display for Tag {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl fmt::Display for Payload {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Payload::Byte(byte) => write!(f, "{}", byte)?,
            Payload::Short(short) => write!(f, "{}", short)?,
            Payload::Int(int) => write!(f, "{}", int)?,
            Payload::Long(long) => write!(f, "{}", long)?,
            Payload::Float(float) => write!(f, "{}", float)?,
            Payload::Double(double) => write!(f, "{}", double)?,
            Payload::String(string) => write!(f, "\"{}\"", string.value)?,
            Payload::List(list) => write!(f, "{}", list)?,
            Payload::Compound(compound) => write!(f, "{}", compound)?,
            Payload::IntArray(int_array) => write!(f, "{}", int_array)?,
            Payload::LongArray(long_array) => write!(f, "{}", long_array)?,
        }
        Ok(())
    }
}