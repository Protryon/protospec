use super::*;

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
