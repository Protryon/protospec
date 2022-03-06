use super::*;

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
fn test_compiler_little_endian() {
    let asg = load_asg(
        r#"
        type test_le = container {
            f1: u32le,
            f2: u64le,
        };
        type test_be = container {
            f1: u32,
            f2: u64,
        };
    "#,
    )
    .unwrap();

    let test = quote! {
        {
            let item = test_le { f1: 100, f2: 200 };
            let mut out = vec![];
            item.encode_sync(&mut out).expect("failed to encode");
            let decoded = test_le::decode_sync(&mut &out[..]).expect("failed to decode");
            assert_eq!(item, decoded);
            let decoded_be = test_be::decode_sync(&mut &out[..]).expect("failed to decode");
            assert_eq!(item.f1, decoded_be.f1.swap_bytes());
            assert_eq!(item.f2, decoded_be.f2.swap_bytes());
        }
    };

    compile("little_endian", &compile_test_program(&asg, test));
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
