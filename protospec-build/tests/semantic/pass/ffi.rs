use crate::*;

#[test]
fn test_ffi_type() {
    load_asg(
        r#"
    import_ffi test_type as type;

    type test = test_type[3];

    type test_into = container {
        int: u32,
        stuff: u8[3] { (int :> test_type) > 3u32 },
    };
    "#,
    )
    .unwrap();
}

#[test]
fn test_ffi_transform() {
    load_asg(
        r#"
    import_ffi test_transform as transform;

    type x = u32 -> test_transform;
    "#,
    )
    .unwrap();
}
