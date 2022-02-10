use crate::*;

#[test]
fn test_ffi_type() {
    parse(
        r#"
    import_ffi test as type;
    "#,
    )
    .unwrap();
}

#[test]
fn test_ffi_transform() {
    parse(
        r#"
    import_ffi test as transform;
    "#,
    )
    .unwrap();
}
