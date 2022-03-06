use super::*;

#[test]
fn test_compiler_calculated() {
    let asg = load_asg(
        r#"
        type Payload = container {
            child_count: u16 = len(children_windows) :> u16,
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

    compile("calculated", &compile_test_program(&asg, test));
}

#[test]
fn test_compiler_calculated_blen() {
    let asg = load_asg(
        r#"
        type Payload = container {
            length: u16 = blen(windows) :> u16,
            windows: container [length] {
                count: u16 = len(windows) :> u16,
                windows: u32[count],
            }
        };
    "#,
    )
    .unwrap();

    let test = quote! {
        fn roundtrip(mut item: Payload) {
            let mut out = vec![];
            item.encode_sync(&mut out).expect("failed to encode");
            let decoded = Payload::decode_sync(&mut &out[..]).expect("failed to decode");
            // item.child_count = item.children_windows.len() as u16;
            item.length = item.windows.len() as u16 * 4 + 2;
            item.count = item.windows.len() as u16;
            assert_eq!(&item, &decoded);
        }
        roundtrip(Payload {
            length: 0,
            count: 0,
            windows: vec![12u32, 19939393u32],
        });
    };

    compile("calculated_blen", &compile_test_program(&asg, test));
}

