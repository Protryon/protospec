#[macro_use]
extern crate quote;

#[macro_use]
pub mod result;
use std::{
    io::Write,
    path::PathBuf,
    process::{Command, Stdio},
};

pub use result::*;

pub mod tokenizer;
pub use tokenizer::*;

pub mod ast;
pub use ast::*;

pub mod asg;
pub use asg::*;

pub mod parser;
pub use parser::*;

pub mod semantics;
pub use semantics::*;

pub mod import;
pub use import::*;

pub mod compiler;
pub use compiler::*;

pub mod coder;

pub mod prelude;
pub use prelude::*;

pub mod ffi;
pub use ffi::*;

#[derive(Clone)]
pub struct Options<T: ImportResolver + 'static> {
    pub format_output: bool,
    pub enum_derives: Vec<String>,
    pub struct_derives: Vec<String>,
    pub include_async: bool,
    pub use_anyhow: bool,
    pub debug_mode: bool,
    pub resolver: T
}

impl Default for Options<NullImportResolver> {
    fn default() -> Self {
        Options {
            format_output: true,
            include_async: false,
            debug_mode: false,
            enum_derives: vec![
                "Eq".to_string(),
                "PartialEq".to_string(),
                "Debug".to_string(),
                "Clone".to_string(),
                "Default".to_string(),
            ],
            struct_derives: vec![
                "Eq".to_string(),
                "PartialEq".to_string(),
                "Debug".to_string(),
                "Clone".to_string(),
                "Default".to_string(),
            ],
            use_anyhow: false,
            resolver: NullImportResolver,
        }
    }
}

pub fn rustfmt(input: &str) -> String {
    let mut proc = Command::new("rustfmt")
        .arg("--edition")
        .arg("2021")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::inherit())
        .spawn()
        .expect("rustfmt failed");
    let stdin = proc.stdin.as_mut().unwrap();
    stdin.write_all(input.as_bytes()).unwrap();
    String::from_utf8_lossy(&proc.wait_with_output().unwrap().stdout).to_string()
}

pub fn compile_spec<T: ImportResolver + Clone + 'static>(
    name: &str,
    spec: &str,
    options: &Options<T>,
) -> AsgResult<()> {
    let resolver = PreludeImportResolver(options.resolver.clone());
    let program =
        asg::Program::from_ast(&parse(spec).map_err(|x| -> Error { x.into() })?, &resolver)?;
    let compiler_options = CompileOptions {
        enum_derives: options.enum_derives.clone(),
        struct_derives: options.struct_derives.clone(),
        include_async: options.include_async,
        use_anyhow: options.use_anyhow,
        debug_mode: options.debug_mode,
    };
    let compiled = compiler::compile_program(&program, &compiler_options);
    let mut compiled = compiled.to_string();
    if options.format_output {
        compiled = rustfmt(&compiled);
    }
    let mut target: PathBuf = std::env::var("OUT_DIR")
        .expect("OUT_DIR env var not set")
        .into();
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
