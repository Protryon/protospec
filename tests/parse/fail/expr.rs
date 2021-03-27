use crate::*;

#[test]
fn test_int_broken() {
    parse(
        r#"
    const _: u32 = 5x5;
    "#,
    )
    .err()
    .unwrap();
}

#[test]
fn test_string_hang() {
    parse(
        r#"
    const _: u32 = "te\";
    "#,
    )
    .err()
    .unwrap();
}

#[test]
fn test_bad_ident() {
    parse(
        r#"
    const _: u32 = $test;
    "#,
    )
    .err()
    .unwrap();
}

#[test]
fn test_paren_hang() {
    parse(
        r#"
    const _: u32 = (5;
    "#,
    )
    .err()
    .unwrap();
}

#[test]
fn test_array_index_empty() {
    parse(
        r#"
    const _: u32 = test[];
    "#,
    )
    .err()
    .unwrap();
}

#[test]
fn test_negate_hang() {
    parse(
        r#"
    const _: u32 = -;
    "#,
    )
    .err()
    .unwrap();
}

#[test]
fn test_not_hang() {
    parse(
        r#"
    const _: u32 = !!;
    "#,
    )
    .err()
    .unwrap();
}

#[test]
fn test_or_hang() {
    parse(
        r#"
    const _: u32 = true ||;
    "#,
    )
    .err()
    .unwrap();
}

#[test]
fn test_ternary_unterm() {
    parse(
        r#"
    const _: u32 = true ? false ? 3 : 2;
    "#,
    )
    .err()
    .unwrap();
}
