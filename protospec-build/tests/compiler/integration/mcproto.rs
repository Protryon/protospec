use super::*;

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
