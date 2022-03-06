use super::*;

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
fn test_compiler_container_pad() {
    let asg = load_asg(
        r#"
    type test = container {
        x: u32,
        .pad: 4,
        y: u32
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
        roundtrip(&test {
            x: 10,
            y: 20
        });
    };

    compile("container_pad", &compile_test_program(&asg, test));
}

#[test]
fn test_compiler_container_multi_nested() {
    let asg = load_asg(
        r#"
    type Interior = container {
        value: u32,
    };
    type Test = container {
        .pad: 4,
        x: bool,
        types: container {
            .pad: 4,
            length: u32 = blen(types) :> u32,
            types: container [length] {
                types: Interior,
            },
        } { x },
    };
    "#,
    )
    .unwrap();
    let test = quote! {
        fn roundtrip(item: &Test) {
            let mut out = vec![];
            item.encode_sync(&mut out).expect("failed to encode");
            let decoded = Test::decode_sync(&mut &out[..]).expect("failed to decode");
            assert_eq!(item, &decoded);
        }
        roundtrip(&Test {
            x: true,
            length: Some(4),
            types: Some(Interior {
                value: 20,
            }),
            ..Default::default()
        });
    };

    compile("container_multi_nested", &compile_test_program(&asg, test));
}
