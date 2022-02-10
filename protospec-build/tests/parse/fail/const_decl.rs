use crate::*;

#[test]
fn test_const_no_type() {
    parse(
        r#"
    const TEST = 5;
    "#,
    )
    .err()
    .unwrap();
}

#[test]
fn test_const_empty_type() {
    parse(
        r#"
    const TEST: = 5;
    "#,
    )
    .err()
    .unwrap();
}

#[test]
fn test_const_invalid_type() {
    parse(
        r#"
    const TEST: type = 5;
    "#,
    )
    .err()
    .unwrap();
}

#[test]
fn test_const_no_value() {
    parse(
        r#"
    const TEST: u32 =;
    "#,
    )
    .err()
    .unwrap();
}

#[test]
fn test_const_no_eq() {
    parse(
        r#"
    const TEST: u32 5;
    "#,
    )
    .err()
    .unwrap();
}

#[test]
fn test_const_bad_ident() {
    parse(
        r#"
    const $TEST: u32 = 5;
    "#,
    )
    .err()
    .unwrap();
}

#[test]
fn test_const_transform() {
    parse(
        r#"
    const TEST: u32 -> gzip = "test";
    "#,
    )
    .err()
    .unwrap();
}
