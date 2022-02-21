use crate::*;
use proc_macro2::TokenStream;
use protospec_build::asg::Program;
use quote::quote;
use std::io::Write;
use std::process::Command;

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
        pub type test_type = Box<u32>;
        #compiled
        fn main() {
            #test
        }
    };
    compiled_test.to_string()
}

#[test]
fn test_compiler_container() {
    let asg = load_asg(
        r#"
    type test = container {
        len: u32,
        is_present: bool,
        data: u8[len] {is_present},
    };
    "#,
    )
    .unwrap();
    let test = quote! {
        fn roundtrip(item: &test) {
            let mut out = vec![];
            item.encode_sync(&mut out).expect("failed to encode");
            let decoded = test::decode_sync(&mut &out[..]).expect("failed to decode");
            assert_eq!(item, &decoded);
        }
        fn encode_fail(item: &test) {
            let mut out = vec![];
            item.encode_sync(&mut out).err().expect("failed to fail encode");
        }
        roundtrip(&test {
            len: 5,
            is_present: true,
            data: Some(vec![0u8, 3u8, 5u8, 1u8, 4u8]),
        });
        roundtrip(&test {
            len: 5,
            is_present: false,
            data: None,
        });
        roundtrip(&test {
            len: 0,
            is_present: true,
            data: Some(vec![]),
        });
        // encode_fail(&test {
        //     len: 1,
        //     is_present: true,
        //     data: Some(vec![]),
        // });
    };

    compile("container", &compile_test_program(&asg, test));
}

#[test]
fn test_compiler_enum() {
    let asg = load_asg(
        r#"
    type test = enum u32 {
        v1 = 1,
        v2,
        v4 = 4,
    };
    "#,
    )
    .unwrap();

    let test = quote! {
        fn roundtrip(item: &test) {
            let mut out = vec![];
            item.encode_sync(&mut out).expect("failed to encode");
            let decoded = test::decode_sync(&mut &out[..]).expect("failed to decode");
            assert_eq!(item, &decoded);
        }
        fn encode_fail(item: &test) {
            let mut out = vec![];
            item.encode_sync(&mut out).err().expect("failed to fail encode");
        }
        roundtrip(&test::v1);
        roundtrip(&test::v2);
        roundtrip(&test::v4);
        // encode_fail(&test {
        //     len: 1,
        //     is_present: true,
        //     data: Some(vec![]),
        // });
    };

    compile("enum", &compile_test_program(&asg, test));
}

#[test]
fn test_compiler_bitfield() {
    let asg = load_asg(
        r#"
    type test = bitfield u32 {
        v1 = 0x01,
        v2,
        v4 = 0x10,
        v9 = 0x100,
    };
    "#,
    )
    .unwrap();

    let test = quote! {
        fn roundtrip(item: test) {
            let mut out = vec![];
            item.encode_sync(&mut out).expect("failed to encode");
            let decoded = test::decode_sync(&mut &out[..]).expect("failed to decode");
            assert_eq!(item, decoded);
        }
        roundtrip(test::V1 | test::V2);
        assert_eq!(test::decode_sync(&mut &0u32.to_be_bytes()[..]).unwrap(), test(0));
        assert_eq!(test::decode_sync(&mut &2u32.to_be_bytes()[..]).unwrap(), test::V2);
        assert_eq!(test::decode_sync(&mut &3u32.to_be_bytes()[..]).unwrap(), test::V2 | test::V1);
        assert_eq!(test::decode_sync(&mut &0x101u32.to_be_bytes()[..]).unwrap(), test::V9 | test::V1);
        test::decode_sync(&mut &0x200u32.to_be_bytes()[..]).err().unwrap();
    };

    compile("bitfield", &compile_test_program(&asg, test));
}

#[test]
fn test_compiler_bitfield_member() {
    let asg = load_asg(
        r#"
    type flags = bitfield u32 {
        A = 0x1,
        B,
        C,
    };
    type test = container {
        bitmask: flags,
        a_value: u8 { bitmask.A },
        b_value: u16 { bitmask.B },
        c_value: u32 { bitmask.C },
    };
    "#,
    )
    .unwrap();

    let test = quote! {
        fn roundtrip(item: test) {
            let mut out = vec![];
            item.encode_sync(&mut out).expect("failed to encode");
            let decoded = test::decode_sync(&mut &out[..]).expect("failed to decode");
            assert_eq!(item, decoded);
        }
        roundtrip(test {
            bitmask: flags::A,
            a_value: Some(5u8),
            b_value: None,
            c_value: None,
        });
        roundtrip(test {
            bitmask: flags::A | flags::C,
            a_value: Some(5u8),
            b_value: None,
            c_value: Some(10),
        });
    };

    compile("bitfield_member", &compile_test_program(&asg, test));
}

#[test]
fn test_compiler_primitive() {
    let asg = load_asg(
        r#"
    type ux32 = u32;
    type ix32 = i32;
    type fx32 = f32;
    type bol = bool;
    "#,
    )
    .unwrap();

    let test = quote! {
        {
            let item: ux32 = ux32(5u32);
            let mut out = vec![];
            item.encode_sync(&mut out).expect("failed to encode");
            let decoded = ux32::decode_sync(&mut &out[..]).expect("failed to decode");
            assert_eq!(item, decoded);
        }
    };

    compile("primitive", &compile_test_program(&asg, test));
}

#[test]
fn test_compiler_var_primitive() {
    let asg = load_asg(
        r#"
    import_ffi v32 as type;

    type x32 = v32;
    "#,
    )
    .unwrap();

    let test = quote! {
        {
            let item = x32(5i32);
            let mut out = vec![];
            item.encode_sync(&mut out).expect("failed to encode");
            let decoded = x32::decode_sync(&mut &out[..]).expect("failed to decode");
            assert_eq!(item, decoded);
            assert_eq!(out.len(), 1);
        }
    };

    compile("var_primitive", &compile_test_program(&asg, test));
}

#[test]
fn test_compiler_container_ref() {
    let asg = load_asg(
        r#"
    type tester = container {
        len: u32,
        is_present: bool,
        data: u8[len] {is_present},
    };

    type test = tester;
    "#,
    )
    .unwrap();
    let test = quote! {
        fn roundtrip(item: &test) {
            let mut out = vec![];
            item.encode_sync(&mut out).expect("failed to encode");
            let decoded = test::decode_sync(&mut &out[..]).expect("failed to decode");
            assert_eq!(item, &decoded);
        }
        fn encode_fail(item: &test) {
            let mut out = vec![];
            item.encode_sync(&mut out).err().expect("failed to fail encode");
        }
        roundtrip(&test(tester {
            len: 5,
            is_present: true,
            data: Some(vec![0u8, 3u8, 5u8, 1u8, 4u8]),
        }));
        roundtrip(&test(tester {
            len: 5,
            is_present: false,
            data: None,
        }));
        roundtrip(&test(tester {
            len: 0,
            is_present: true,
            data: Some(vec![]),
        }));
        // encode_fail(&test {
        //     len: 1,
        //     is_present: true,
        //     data: Some(vec![]),
        // });
    };

    compile("container_ref", &compile_test_program(&asg, test));
}

#[test]
fn test_compiler_container_length() {
    let asg = load_asg(
        r#"
    type tester = container [190] {
        len: u32,
        is_present: bool,
        data: u8[..] {is_present},
    };

    type test = tester;
    "#,
    )
    .unwrap();

    let test = quote! {
        fn roundtrip(item: &test) {
            let mut out = vec![];
            item.encode_sync(&mut out).expect("failed to encode");
            let decoded = test::decode_sync(&mut &out[..]).expect("failed to decode");
            assert_eq!(item, &decoded);
        }
        roundtrip(&test(tester {
            len: 5,
            is_present: true,
            data: Some(vec![0u8; 190 - 5]),
        }));
        // roundtrip(&test(tester {
        //     len: 5,
        //     is_present: false,
        //     data: None,
        // }));
        // roundtrip(&test(tester {
        //     len: 0,
        //     is_present: true,
        //     data: Some(vec![]),
        // }));
    };

    compile("container_length", &compile_test_program(&asg, test));
}

#[test]
fn test_compiler_container_fill() {
    let asg = load_asg(
        r#"
    type tester = container {
        len: u32,
        is_present: bool,
        data: u8[..] {is_present},
    };

    type test = tester;
    "#,
    )
    .unwrap();

    let test = quote! {
        fn roundtrip(item: &test) {
            let mut out = vec![];
            item.encode_sync(&mut out).expect("failed to encode");
            let decoded = test::decode_sync(&mut &out[..]).expect("failed to decode");
            assert_eq!(item, &decoded);
        }
        roundtrip(&test(tester {
            len: 5,
            is_present: true,
            data: Some(vec![0u8, 3u8, 5u8, 1u8, 4u8]),
        }));
        roundtrip(&test(tester {
            len: 5,
            is_present: false,
            data: None,
        }));
        roundtrip(&test(tester {
            len: 0,
            is_present: true,
            data: Some(vec![]),
        }));
    };

    compile("container_fill", &compile_test_program(&asg, test));
}

#[test]
fn test_compiler_container_repeated() {
    let asg = load_asg(
        r#"
    type tester = container {
        len: u32,
        is_present: bool,
        data: u8[len] {is_present},
    };

    type test = tester[3];
    "#,
    )
    .unwrap();

    let test = quote! {
        fn roundtrip(item: &test) {
            let mut out = vec![];
            item.encode_sync(&mut out).expect("failed to encode");
            let decoded = test::decode_sync(&mut &out[..]).expect("failed to decode");
            assert_eq!(item, &decoded);
        }
        roundtrip(&test(vec![tester {
            len: 5,
            is_present: true,
            data: Some(vec![0u8, 3u8, 5u8, 1u8, 4u8]),
        }; 3]));
    };

    compile("container_repeated", &compile_test_program(&asg, test));
}

#[test]
fn test_compiler_container_eof() {
    let asg = load_asg(
        r#"
    type tester = container {
        len: u32,
        is_present: bool,
        data: u8[len] {is_present},
    };

    type test = tester[..];
    "#,
    )
    .unwrap();

    let test = quote! {
        fn roundtrip(item: &test) {
            let mut out = vec![];
            item.encode_sync(&mut out).expect("failed to encode");
            let decoded = test::decode_sync(&mut &out[..]).expect("failed to decode");
            assert_eq!(item, &decoded);
        }
        roundtrip(&test(vec![tester {
            len: 5,
            is_present: true,
            data: Some(vec![0u8, 3u8, 5u8, 1u8, 4u8]),
        }; 5]));
    };

    compile("container_eof", &compile_test_program(&asg, test));
}

#[test]
fn test_compiler_transform() {
    let asg = load_asg(
        r#"
    import_ffi test_transform as transform;
    
    type tester = container {
        len: u32,
        is_present: bool,
        data: u8[len] {is_present} -> test_transform,
    };

    type test = tester;
    "#,
    )
    .unwrap();

    let test = quote! {
        fn roundtrip(item: &test) {
            let mut out = vec![];
            item.encode_sync(&mut out).expect("failed to encode");
            let decoded = test::decode_sync(&mut &out[..]).expect("failed to decode");
            assert_eq!(item, &decoded);
        }
        roundtrip(&test(tester {
            len: 5,
            is_present: true,
            data: Some(vec![0u8, 3u8, 5u8, 1u8, 4u8]),
        }));
        roundtrip(&test(tester {
            len: 5,
            is_present: false,
            data: None,
        }));
        roundtrip(&test(tester {
            len: 0,
            is_present: true,
            data: Some(vec![]),
        }));
    };

    compile("transform", &compile_test_program(&asg, test));
}

#[test]
fn test_compiler_transform_conditional() {
    let asg = load_asg(
        r#"
    import_ffi test_transform as transform;
    
    type tester = container {
        len: u32,
        is_present: bool,
        is_base64: bool,
        data: u8[len] {is_present} -> test_transform {is_base64},
    };

    type test = tester;
    "#,
    )
    .unwrap();

    let test = quote! {
        fn roundtrip(item: &test) {
            let mut out = vec![];
            item.encode_sync(&mut out).expect("failed to encode");
            let decoded = test::decode_sync(&mut &out[..]).expect("failed to decode");
            assert_eq!(item, &decoded);
        }
        roundtrip(&test(tester {
            len: 5,
            is_present: true,
            is_base64: true,
            data: Some(vec![0u8, 3u8, 5u8, 1u8, 4u8]),
        }));
        roundtrip(&test(tester {
            len: 5,
            is_present: true,
            is_base64: false,
            data: Some(vec![0u8, 3u8, 5u8, 1u8, 4u8]),
        }));
        roundtrip(&test(tester {
            len: 5,
            is_present: false,
            is_base64: true,
            data: None,
        }));
        roundtrip(&test(tester {
            len: 0,
            is_present: true,
            is_base64: true,
            data: Some(vec![]),
        }));
    };

    compile("transform_conditional", &compile_test_program(&asg, test));
}

#[test]
fn test_compiler_transform_conditional_arg() {
    let asg = load_asg(
        r#"
    import_ffi test_transform as transform;
    
    type tester = container {
        len: u32,
        is_present: bool,
        is_base64: bool,
        data: u8[len] {is_present} -> test_transform(2) {is_base64},
    };

    type test = tester;
    "#,
    )
    .unwrap();

    let test = quote! {
        fn roundtrip(item: &test) {
            let mut out = vec![];
            item.encode_sync(&mut out).expect("failed to encode");
            let decoded = test::decode_sync(&mut &out[..]).expect("failed to decode");
            assert_eq!(item, &decoded);
        }
        roundtrip(&test(tester {
            len: 5,
            is_present: true,
            is_base64: true,
            data: Some(vec![0u8, 3u8, 5u8, 1u8, 4u8]),
        }));
        roundtrip(&test(tester {
            len: 5,
            is_present: true,
            is_base64: false,
            data: Some(vec![0u8, 3u8, 5u8, 1u8, 4u8]),
        }));
        roundtrip(&test(tester {
            len: 5,
            is_present: false,
            is_base64: true,
            data: None,
        }));
        roundtrip(&test(tester {
            len: 0,
            is_present: true,
            is_base64: true,
            data: Some(vec![]),
        }));
    };

    compile(
        "transform_conditional_arg",
        &compile_test_program(&asg, test),
    );
}

#[test]
fn test_compiler_type_args() {
    let asg = load_asg(
        r#"
    import_ffi test_transform as transform;
    
    type test(is_base64: bool) = container {
        len: u32,
        is_present: bool,
        data: u8[len] {is_present} -> test_transform(2) {is_base64},
    };

    type tester = test(false);
    "#,
    )
    .unwrap();

    let test = quote! {
        fn roundtrip(item: &test) {
            let mut out = vec![];
            item.encode_sync(&mut out, true).expect("failed to encode");
            let decoded = test::decode_sync(&mut &out[..], true).expect("failed to decode");
            assert_eq!(item, &decoded);
        }
        fn roundtrip2(item: &tester) {
            let mut out = vec![];
            item.encode_sync(&mut out).expect("failed to encode");
            let decoded = tester::decode_sync(&mut &out[..]).expect("failed to decode");
            assert_eq!(item, &decoded);
        }
        roundtrip(&test {
            len: 5,
            is_present: true,
            data: Some(vec![0u8, 3u8, 5u8, 1u8, 4u8]),
        });
        roundtrip(&test {
            len: 5,
            is_present: true,
            data: Some(vec![0u8, 3u8, 5u8, 1u8, 4u8]),
        });
        roundtrip(&test {
            len: 5,
            is_present: false,
            data: None,
        });
        roundtrip2(&tester(test {
            len: 5,
            is_present: false,
            data: None,
        }));
        roundtrip(&test {
            len: 0,
            is_present: true,
            data: Some(vec![]),
        });
    };

    compile("type_args", &compile_test_program(&asg, test));
}

#[test]
fn test_compiler_foreign_type() {
    let asg = load_asg(
        r#"
    import_ffi test_type as type;
    
    type tester = container {
        len: u32,
        is_present: bool,
        data: test_type[len] {is_present},
    };

    type test = tester;
    "#,
    )
    .unwrap();

    let test = quote! {
        fn roundtrip(item: &test) {
            let mut out = vec![];
            item.encode_sync(&mut out).expect("failed to encode");
            let decoded = test::decode_sync(&mut &out[..]).expect("failed to decode");
            assert_eq!(item, &decoded);
        }
        roundtrip(&test(tester {
            len: 2,
            is_present: true,
            data: Some(vec![Box::new(0u32), Box::new(7u32)]),
        }));
        roundtrip(&test(tester {
            len: 5,
            is_present: false,
            data: None,
        }));
        roundtrip(&test(tester {
            len: 0,
            is_present: true,
            data: Some(vec![]),
        }));
    };

    compile("foreign_type", &compile_test_program(&asg, test));
}

#[test]
fn test_compiler_expr() {
    let asg = load_asg(
        r#"
    type test = u32[1 + 2];

    type tester = test;
    "#,
    )
    .unwrap();

    let test = quote! {
        fn roundtrip(item: &test) {
            let mut out = vec![];
            item.encode_sync(&mut out).expect("failed to encode");
            let decoded = test::decode_sync(&mut &out[..]).expect("failed to decode");
            assert_eq!(item, &decoded);
        }
        roundtrip(&test(vec![0, 1, 2]));
        roundtrip(&test(vec![2, 1, 0]));
        roundtrip(&test(vec![2, 3, 0]));
    };

    compile("compiler_expr", &compile_test_program(&asg, test));
}

#[test]
fn test_compiler_mcproto() {
    let asg = load_asg(
        r#"
        import_ffi v32 as type;
        import_ffi gzip as transform;
        
        type Packet(is_compressed: bool) = container {
            length: v32 +auto,
            packet: container [length] {
                uncompressed_length: v32 {is_compressed},
                compressable: container {
                    id: v32,
                    data: u8[..],
                } -> gzip {is_compressed && uncompressed_length?:0i32 > 0i32},
            },
        };
    "#,
    )
    .unwrap();

    let test = quote! {
        // fn roundtrip(item: &test) {
        //     let mut out = vec![];
        //     item.encode_sync(&mut out).expect("failed to encode");
        //     let decoded = test::decode_sync(&out[..]).expect("failed to decode");
        //     assert_eq!(item, &decoded);
        // }
        // roundtrip(&test(vec![0, 1, 2]));
        // roundtrip(&test(vec![2, 1, 0]));
        // roundtrip(&test(vec![2, 3, 0]));
    };

    compile("mcproto", &compile_test_program(&asg, test));
}

#[test]
fn test_compiler_png() {
    let asg = load_asg(
        r#"
        type PngChunk = container {
            length: u32,
            chunk_type: u32,
            data: u8[length],
            crc: u32,
        };
        type Png = container {
            header: u8[8],
            chunks: PngChunk[..]
        };
    "#,
    )
    .unwrap();

    let test = quote! {
        // fn roundtrip(item: &test) {
        //     let mut out = vec![];
        //     item.encode_sync(&mut out).expect("failed to encode");
        //     let decoded = test::decode_sync(&out[..]).expect("failed to decode");
        //     assert_eq!(item, &decoded);
        // }
        // roundtrip(&test(vec![0, 1, 2]));
        // roundtrip(&test(vec![2, 1, 0]));
        // roundtrip(&test(vec![2, 3, 0]));
    };

    compile("png", &compile_test_program(&asg, test));
}


#[test]
fn test_compiler_tagged_enum() {
    let asg = load_asg(
        r#"
        type Payload(t: u8) = container +tagged_enum {
            Byte: i8 {t == 1},
            Short: i16 {t == 2},
        };
        type Outer = container {
            tag: u8,
            payload: Payload(tag),
        };
    "#,
    )
    .unwrap();

    let test = quote! {
        fn roundtrip(item: &Outer) {
            let mut out = vec![];
            item.encode_sync(&mut out).expect("failed to encode");
            let decoded = Outer::decode_sync(&mut &out[..]).expect("failed to decode");
            assert_eq!(item, &decoded);
        }
        roundtrip(&Outer {
            tag: 1,
            payload: Payload::Byte(7),
        });
        roundtrip(&Outer {
            tag: 2,
            payload: Payload::Short(7),
        });
    };

    compile("tagged_enum", &compile_test_program(&asg, test));
}

#[test]
fn test_compiler_tagged_enum_default() {
    let asg = load_asg(
        r#"
        type Payload(t: u8) = container +tagged_enum {
            Byte: i8 {t == 1},
            Short: i16 {t == 2},
            Other: u32,
        };
        type Outer = container {
            tag: u8,
            payload: Payload(tag),
        };
    "#,
    )
    .unwrap();

    let test = quote! {
        fn roundtrip(item: &Outer) {
            let mut out = vec![];
            item.encode_sync(&mut out).expect("failed to encode");
            let decoded = Outer::decode_sync(&mut &out[..]).expect("failed to decode");
            assert_eq!(item, &decoded);
        }
        roundtrip(&Outer {
            tag: 1,
            payload: Payload::Byte(7),
        });
        roundtrip(&Outer {
            tag: 2,
            payload: Payload::Short(9),
        });
        roundtrip(&Outer {
            tag: 3,
            payload: Payload::Other(11),
        });
        roundtrip(&Outer {
            tag: 0,
            payload: Payload::Other(11),
        });
    };

    compile("tagged_enum_default", &compile_test_program(&asg, test));
}

#[test]
fn test_compiler_tagged_enum_struct() {
    let asg = load_asg(
        r#"
        type Payload(t: u8) = container +tagged_enum {
            Byte: container {
                b1: u8,
                b2: u8,
            } {t == 1},
            Short: i16 {t == 2},
        };
        type Outer = container {
            tag: u8,
            payload: Payload(tag),
        };
    "#,
    )
    .unwrap();

    let test = quote! {
        fn roundtrip(item: &Outer) {
            let mut out = vec![];
            item.encode_sync(&mut out).expect("failed to encode");
            let decoded = Outer::decode_sync(&mut &out[..]).expect("failed to decode");
            assert_eq!(item, &decoded);
        }
        roundtrip(&Outer {
            tag: 1,
            payload: Payload::Byte {
                b1: 7,
                b2: 11,
            },
        });
        roundtrip(&Outer {
            tag: 2,
            payload: Payload::Short(9),
        });
    };

    compile("tagged_enum_struct", &compile_test_program(&asg, test));
}

#[test]
fn test_compiler_tagged_nested_enum_struct() {
    let asg = load_asg(
        r#"
        type Payload(t: u8) = container +tagged_enum {
            Byte: container {
                b1: u8,
                b2: u8,
                b3: container {
                    b4: u8,
                }
            } {t == 1},
            Short: i16 {t == 2},
        };
        type Outer = container {
            tag: u8,
            payload: Payload(tag),
        };
    "#,
    )
    .unwrap();

    let test = quote! {
        fn roundtrip(item: &Outer) {
            let mut out = vec![];
            item.encode_sync(&mut out).expect("failed to encode");
            let decoded = Outer::decode_sync(&mut &out[..]).expect("failed to decode");
            assert_eq!(item, &decoded);
        }
        roundtrip(&Outer {
            tag: 1,
            payload: Payload::Byte {
                b1: 7,
                b2: 11,
                b4: 13,
            },
        });
        roundtrip(&Outer {
            tag: 2,
            payload: Payload::Short(9),
        });
    };

    compile("tagged_nested_enum_struct", &compile_test_program(&asg, test));
}

#[test]
fn test_compiler_auto_distant() {
    let asg = load_asg(
        r#"
        type Payload = container {
            child_count: u16 +auto,
            reserved: u8[1],
            children_windows: u32[child_count],
        };
    "#,
    )
    .unwrap();

    let test = quote! {
        fn roundtrip(mut item: Payload) {
            let mut out = vec![];
            item.encode_sync(&mut out).expect("failed to encode");
            let decoded = Payload::decode_sync(&mut &out[..]).expect("failed to decode");
            item.child_count = item.children_windows.len() as u16;
            assert_eq!(&item, &decoded);
        }
        roundtrip(Payload {
            child_count: 0,
            reserved: vec![0u8],
            children_windows: vec![5u32],
        });
    };

    compile("auto_distant", &compile_test_program(&asg, test));
}


#[test]
fn test_compiler_integration1() {
    let asg = load_asg(
        r#"
        import_ffi utf8 as type;

        type Packet = container {
            id : u8,
            size: u32 +auto,
            size_crc: u8,
            data: container [size] {
                data: Item[..],
            },
            crc: u8,
        };

        type TypeId = enum u8 {
            String = 0,
            Integer,
            Float,
            Long,
        };

        type Item = container {
            code: u16,
            type_id: TypeId,
            body: ItemBody(type_id),
        };

        type ItemBody(type_id: TypeId) = container +tagged_enum {
            String: container {
                len: u32 +auto,
                string: utf8(len),
            } { type_id == TypeId::String },
            Integer: u32 { type_id == TypeId::Integer },
            Float: u32 { type_id == TypeId::Float },
            Long: u32 { type_id == TypeId::Long },
        };
    "#,
    )
    .unwrap();

    let test = quote! {
        // fn roundtrip(mut item: Payload) {
        //     let mut out = vec![];
        //     item.encode_sync(&mut out).expect("failed to encode");
        //     let decoded = Payload::decode_sync(&mut &out[..]).expect("failed to decode");
        //     item.child_count = item.children_windows.len() as u16;
        //     assert_eq!(&item, &decoded);
        // }
        // roundtrip(Payload {
        //     child_count: 0,
        //     reserved: vec![0u8],
        //     children_windows: vec![5u32],
        // });
    };

    compile("integration1", &compile_test_program(&asg, test));
}
