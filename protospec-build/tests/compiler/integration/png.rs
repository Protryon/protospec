use super::*;

#[test]
fn test_compiler_png() {
    let asg = load_asg(
        r#"
        type PngChunk = container {
            length: u32,
            chunk_type: u32,
            data: u8[length],
            crc: u32,
        };
        type Png = container {
            header: u8[8],
            chunks: PngChunk[..]
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

    compile("png", &compile_test_program(&asg, test));
}
