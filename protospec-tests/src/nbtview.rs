
pub mod nbt {
    protospec::include_spec!("nbt");
}
use nbt::*;

use std::{io::BufReader, fs::File, fmt, fmt::Write};

use clap::Parser;
use indenter::indented;

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    input: String,
}

fn main() {
    let args = Args::parse();
    let file = File::open(&*args.input).expect("failed to open file");
    let mut file_reader = BufReader::new(file);
    let nbt = nbt::Compound::decode_sync(&mut file_reader).expect("failed to decode nbt");
    println!("{}", nbt);
}

impl fmt::Display for Compound {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for item in &self.items {
            write!(indented(f).with_str("  "), "\n{}<{}> = {}", item.type_id, String::from_utf8_lossy(&*item.name), item.payload)?;
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
            Payload { byte: Some(byte), .. } => write!(f, "{}", byte)?,
            Payload { short: Some(short), .. } => write!(f, "{}", short)?,
            Payload { int: Some(int), .. } => write!(f, "{}", int)?,
            Payload { long: Some(long), .. } => write!(f, "{}", long)?,
            Payload { float: Some(float), .. } => write!(f, "{}", float)?,
            Payload { double: Some(double), .. } => write!(f, "{}", double)?,
            Payload { string: Some(string), .. } => write!(f, "\"{}\"", String::from_utf8_lossy(&*string.value))?,
            Payload { list: Some(list), .. } => write!(f, "{}", list)?,
            Payload { compound: Some(compound), .. } => write!(f, "{}", compound)?,
            Payload { int_array: Some(int_array), .. } => write!(f, "{}", int_array)?,
            Payload { long_array: Some(long_array), .. } => write!(f, "{}", long_array)?,
            _ => unreachable!("invalid payload"),
        }
        Ok(())
    }
}