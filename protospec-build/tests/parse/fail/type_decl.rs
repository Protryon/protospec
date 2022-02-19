use crate::*;

#[test]
fn test_semicolon() {
    parse(
        r#"
    type test = u32
    "#,
    )
    .err()
    .unwrap();
}

#[test]
fn test_no_name() {
    parse(
        r#"
    type = u32;
    "#,
    )
    .err()
    .unwrap();
}

#[test]
fn test_bad_name() {
    parse(
        r#"
    type type = u32;
    "#,
    )
    .err()
    .unwrap();
}

#[test]
fn test_type_undefined() {
    parse(
        r#"
    type t;
    "#,
    )
    .err()
    .unwrap();
}

#[test]
fn test_type_hanging() {
    parse(
        r#"
    type t = ;
    "#,
    )
    .err()
    .unwrap();
}

#[test]
fn test_container_empty_length() {
    parse(
        r#"
    type test = container [] {
        west: u32,
    };
    "#,
    )
    .err()
    .unwrap();
}

#[test]
fn test_enum_no_init() {
    parse(
        r#"
    type test = enum i32 {
        test,
        west,
        east,
        north = 6,
        south,
    };
    "#,
    )
    .err()
    .unwrap();
}

#[test]
fn test_conditional_empty() {
    parse(
        r#"
    type test = container {
        len: u32,
        data: u8[len] { },
    };
    "#,
    )
    .err()
    .unwrap();
}

#[test]
fn test_array_empty() {
    parse(
        r#"
    type test = container {
        data: u8[],
    };
    "#,
    )
    .err()
    .unwrap();
}

#[test]
fn test_transform_hanging() {
    parse(
        r#"
    type test = container {
        data: u8[..] ->,
    };
    "#,
    )
    .err()
    .unwrap();
}

#[test]
fn test_transform2_hanging() {
    parse(
        r#"
    type test = container {
        data: u8[..] -> -> encrypt,
    };
    "#,
    )
    .err()
    .unwrap();
}

#[test]
fn test_transform_conditional_array_hanging() {
    parse(
        r#"
    type test = container {
        len: u32,
        is_present: bool,
        data: u8[6] { is_present } -> gzip -> [..],
    };
    "#,
    )
    .err()
    .unwrap();
}

#[test]
fn test_transform_conditional_transform_empty() {
    parse(
        r#"
    type test = container {
        len: u32,
        is_present: bool,
        is_encrypted: bool,
        data: u8[6] { is_present } -> gzip -> encrypt {},
    };
    "#,
    )
    .err()
    .unwrap();
}
