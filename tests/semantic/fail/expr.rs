use crate::*;

#[test]
fn test_oversize_int() {
    load_asg(
        r#"
    const _: u8 = 257;
    "#,
    )
    .err()
    .unwrap();
}

#[test]
fn test_hex_int_oversize() {
    load_asg(
        r#"
    const _: u8 = 0x5FF;
    "#,
    )
    .err()
    .unwrap();
}

#[test]
fn test_hex_int_neg_unsigned() {
    load_asg(
        r#"
    const _: u32 = -0x5FF;
    "#,
    )
    .err()
    .unwrap();
}

#[test]
fn test_bad_enum_ref() {
    load_asg(
        r#"
    const _: u32 = test.test;
    "#,
    )
    .err()
    .unwrap();
}

#[test]
fn test_bad_ident() {
    load_asg(
        r#"
    const _: u32 = test;
    "#,
    )
    .err()
    .unwrap();
}

#[test]
fn test_array_index_type() {
    load_asg(
        r#"
    const _: u32 = ""[1i32];
    "#,
    )
    .err()
    .unwrap();
}

#[test]
fn test_negate_bool() {
    load_asg(
        r#"
    const _: u32 = -false;
    "#,
    )
    .err()
    .unwrap();
}

#[test]
fn test_not_int() {
    load_asg(
        r#"
    const _: bool = !1;
    "#,
    )
    .err()
    .unwrap();
}

#[test]
fn test_signed_unsigned() {
    load_asg(
        r#"
    const _: u32 = 1i32;
    "#,
    )
    .err()
    .unwrap();
}

#[test]
fn test_mul_bool() {
    load_asg(
        r#"
    const _: u32 = true * 3;
    "#,
    )
    .err()
    .unwrap();
}

#[test]
fn test_mul_bool2() {
    load_asg(
        r#"
    const _: bool = true * 3;
    "#,
    )
    .err()
    .unwrap();
}

#[test]
fn test_or_int() {
    load_asg(
        r#"
    const _: bool = true || 1;
    "#,
    )
    .err()
    .unwrap();
}

#[test]
fn test_ternary() {
    load_asg(
        r#"
    const _: u32 = true ? false ? 3i32 : 2u32 : false;
    "#,
    )
    .err()
    .unwrap();
}
