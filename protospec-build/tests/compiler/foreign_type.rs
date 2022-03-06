use super::*;

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
