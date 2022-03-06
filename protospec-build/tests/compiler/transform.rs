use super::*;

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
