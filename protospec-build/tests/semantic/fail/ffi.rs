use crate::*;

#[test]
fn test_ffi_type_missing() {
    load_asg(
        r#"
    import_ffi test_type2 as type;

    "#,
    )
    .err()
    .unwrap();
}

#[test]
fn test_ffi_type_redefined() {
    load_asg(
        r#"
    type test_type = u32;
    import_ffi test_type as type;

    "#,
    )
    .err()
    .unwrap();
}

#[test]
fn test_ffi_transform_missing() {
    load_asg(
        r#"
    import_ffi test_transform2 as transform;
    "#,
    )
    .err()
    .unwrap();
}

#[test]
fn test_ffi_transform_redefined() {
    load_asg(
        r#"
    import_ffi test_transform as transform;
    import_ffi test_transform as transform;
    "#,
    )
    .err()
    .unwrap();
}

#[test]
fn test_ffi_transform_type() {
    load_asg(
        r#"
    import_ffi test_transform as type;
    "#,
    )
    .err()
    .unwrap();
}

#[test]
fn test_ffi_type_transform() {
    load_asg(
        r#"
    import_ffi test_type as transform;
    "#,
    )
    .err()
    .unwrap();
}
