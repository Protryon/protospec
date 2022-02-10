use super::*;

#[test]
fn test_base() {
    let asg = load_asg(include_str!("./test.pspec")).unwrap();

    let test = quote! {
        let mut input = TestContainer {
            name_len: 0,
            name: "test name".to_string(),
            int1: Some(128),
        };

        let mut out = vec![];
        input.encode_sync(&mut out).expect("failed to encode");
        let decoded = TestContainer::decode_sync(&mut &out[..]).expect("failed to decode");
        assert!(decoded.int1.is_none());
        input.name = "test name-test name".to_string();
        out = vec![];
        input.encode_sync(&mut out).expect("failed to encode");
        let decoded = TestContainer::decode_sync(&mut &out[..]).expect("failed to decode");
        assert!(decoded.int1.is_some());
    };

    compile("test", &compile_test_program(&asg, test));
}
