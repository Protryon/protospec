use super::*;

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

    compile("expr", &compile_test_program(&asg, test));
}

#[test]
fn test_compiler_expr_sum() {
    let asg = load_asg(
        r#"
    import_ffi sum as function;

    type Test = container {
        x: bool,
        kt_levels: container {
            num_levels_per_type: u8[32],
            .pad: 4,
            levels: u32[sum(num_levels_per_type)],
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
        let mut levels = vec![0u8; 32];
        levels[0] = 1;
        levels[1] = 3;
        roundtrip(&Test {
            x: true,
            num_levels_per_type: Some(levels),
            levels: Some(vec![1u32, 2u32, 4u32, 8u32]),
        });
    };

    compile("expr_sum", &compile_test_program(&asg, test));
}
