use crate::*;
use proc_macro2::TokenStream;
use protospec_build::asg::Program;
use quote::quote;
use std::io::Write;
use std::process::Command;

mod container;
mod enum_;
mod bitfield;
mod primitive;
mod transform;
mod foreign_type;
mod expr;
mod tagged_enum;
mod calculated;
mod integration;

pub fn rustfmt(input: &str) -> String {
    let mut proc = Command::new("rustfmt")
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::inherit())
        .spawn()
        .expect("rustfmt failed");
    let stdin = proc.stdin.as_mut().unwrap();
    stdin.write_all(input.as_bytes()).unwrap();
    String::from_utf8_lossy(&proc.wait_with_output().unwrap().stdout).to_string()
}

pub fn lineify(input: &str) -> String {
    input
        .lines()
        .enumerate()
        .map(|(i, line)| format!("{}: {}", i + 1, line))
        .collect::<Vec<_>>()
        .join("\n")
}

pub fn compile(name: &str, input: &str) {
    let input = rustfmt(input);
    println!("{}", lineify(&input));
    let inname = format!("{}_test.rs", name);
    let outname = format!("{}_test.out", name);
    std::fs::write(&inname, &input).expect("failed to write test input file");
    let mut proc = Command::new("rustc")
        .arg(&inname)
        .arg("--crate-name")
        .arg(name)
        .arg("--crate-type")
        .arg("bin")
        .arg("--edition")
        .arg("2018")
        .arg("-o")
        .arg(&outname)
        .stdin(std::process::Stdio::inherit())
        .stdout(std::process::Stdio::inherit())
        .stderr(std::process::Stdio::inherit())
        .spawn()
        .expect("rustc failed");
    let rustc_status = proc.wait().unwrap();
    std::fs::remove_file(&inname).expect("failed to delete input file");
    if !rustc_status.success() {
        std::fs::remove_file(&outname).expect("failed to delete output file");
        panic!("compile failed");
    }
    let mut proc = Command::new(&format!("./{}", outname))
        .stdin(std::process::Stdio::inherit())
        .stdout(std::process::Stdio::inherit())
        .stderr(std::process::Stdio::inherit())
        .spawn()
        .expect("test run failed");
    if !proc.wait().unwrap().success() {
        std::fs::remove_file(&outname).expect("failed to delete output file");
        panic!("test process failed");
    }
    std::fs::remove_file(&outname).expect("failed to delete output file");
}

fn compile_test_program(program: &Program, test: TokenStream) -> String {
    let options = CompileOptions::default();
    let compiled = compiler::compile_program(&program, &options);
    let compiled_test = quote! {
        #compiled
        fn main() {
            #test
        }
    };
    compiled_test.to_string()
}
