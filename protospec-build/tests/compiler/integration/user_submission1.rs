use super::*;

#[test]
fn test_compiler_integration1() {
    let asg = load_asg(
        r#"
        import_ffi utf8 as type;

        type Packet = container {
            id : u8,
            size: u32 = blen(data) :> u32,
            size_crc: u8,
            data: container [size] {
                data: Item[..],
            },
            crc: u8,
        };

        type TypeId = enum u8 {
            String = 0,
            Integer,
            Float,
            Long,
        };

        type Item = container {
            code: u16,
            type_id: TypeId,
            body: ItemBody(type_id),
        };

        type ItemBody(type_id: TypeId) = container +tagged_enum {
            String: container {
                len: u32 = len(string) :> u32,
                string: utf8(len),
            } { type_id == TypeId::String },
            Integer: u32 { type_id == TypeId::Integer },
            Float: u32 { type_id == TypeId::Float },
            Long: u32 { type_id == TypeId::Long },
        };
    "#,
    )
    .unwrap();

    let test = quote! {
        // fn roundtrip(mut item: Payload) {
        //     let mut out = vec![];
        //     item.encode_sync(&mut out).expect("failed to encode");
        //     let decoded = Payload::decode_sync(&mut &out[..]).expect("failed to decode");
        //     item.child_count = item.children_windows.len() as u16;
        //     assert_eq!(&item, &decoded);
        // }
        // roundtrip(Payload {
        //     child_count: 0,
        //     reserved: vec![0u8],
        //     children_windows: vec![5u32],
        // });
    };

    compile("integration1", &compile_test_program(&asg, test));
}
