use super::*;

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
