#[macro_use]
extern crate quote;

#[macro_use]
pub mod result;
use std::{io::Write, path::PathBuf, process::{Command, Stdio}};

pub use result::*;

pub mod tokenizer;
pub use tokenizer::*;

pub mod ast;
pub use ast::*;

pub mod asg;
pub use asg::*;

pub mod parser;
pub use parser::*;

pub mod semantifier;
pub use semantifier::*;

pub mod import;
pub use import::*;

pub mod compiler;
pub use compiler::*;

// pub mod decoder;

pub mod coder;

pub mod prelude;
pub use prelude::*;

#[derive(Clone)]
pub struct Options {
    pub format_output: bool,
    pub derives: Vec<String>,
}

impl Default for Options {
    fn default() -> Self {
        Options {
            format_output: true,
            derives: vec![
                "PartialEq".to_string(),
                "Debug".to_string(),
                "Clone".to_string(),
            ],
        }
    }
}

pub fn rustfmt(input: &str) -> String {
    let mut proc = Command::new("rustfmt")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::inherit())
        .spawn()
        .expect("rustfmt failed");
    let stdin = proc.stdin.as_mut().unwrap();
    stdin.write_all(input.as_bytes()).unwrap();
    String::from_utf8_lossy(&proc.wait_with_output().unwrap().stdout).to_string()
}

pub fn compile_spec(name: &str, spec: &str, options: &Options) -> AsgResult<()> {
    let resolver = PreludeImportResolver(NullImportResolver);
    let program = asg::Program::from_ast(
        &parse(spec).map_err(|x| -> Error { x.into() })?,
        &resolver,
    )?;
    let compiler_options = CompileOptions {
        derives: options.derives.clone(),
    };
    let compiled = compiler::compile_program(&program, &compiler_options);
    let mut compiled = compiled.to_string();
    if options.format_output {
        compiled = rustfmt(&compiled);
    }
    let mut target: PathBuf = std::env::var("OUT_DIR").expect("OUT_DIR env var not set").into();
    target.push(format!("{}.rs", name));
    std::fs::write(target, compiled).expect("failed to write to target");
    Ok(())
}

#[macro_export]
macro_rules! include_spec {
    ($package: tt) => {
        include!(concat!(env!("OUT_DIR"), concat!("/", $package, ".rs")));
    };
}