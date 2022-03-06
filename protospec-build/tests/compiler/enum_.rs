use super::*;

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
