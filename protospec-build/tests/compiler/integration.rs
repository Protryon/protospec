use super::*;

#[test]
fn test_nbt() {
    let asg = load_asg(include_str!("./nbt.pspec")).unwrap();

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

    compile("nbt", &compile_test_program(&asg, test));
}
