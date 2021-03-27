use crate::*;

#[test]
fn test_ffi_type_missing_type() {
    parse(
        r#"
    import_ffi test as;
    "#,
    )
    .err()
    .unwrap();
}

#[test]
fn test_ffi_type_invalid_type() {
    parse(
        r#"
    import_ffi test as test;
    "#,
    )
    .err()
    .unwrap();
}

#[test]
fn test_ffi_type_missing_as() {
    parse(
        r#"
    import_ffi test type;
    "#,
    )
    .err()
    .unwrap();
}

#[test]
fn test_ffi_type_missing_name() {
    parse(
        r#"
    import_ffi as transform;
    "#,
    )
    .err()
    .unwrap();
}
