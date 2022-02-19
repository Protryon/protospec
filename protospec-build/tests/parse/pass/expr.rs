use crate::*;

#[test]
fn test_int() {
    parse(
        r#"
    const _: u32 = 5;
    "#,
    )
    .unwrap();
}

#[test]
fn test_hex_int() {
    parse(
        r#"
    const _: u32 = 0x5FF;
    "#,
    )
    .unwrap();
}

#[test]
fn test_string() {
    parse(
        r#"
    const _: u32 = "test";
    "#,
    )
    .unwrap();
}

#[test]
fn test_string_escaped() {
    parse(
        r#"
    const _: u32 = "te\"st";
    "#,
    )
    .unwrap();
}

#[test]
fn test_ident() {
    parse(
        r#"
    const _: u32 = test;
    "#,
    )
    .unwrap();
}

#[test]
fn test_paren() {
    parse(
        r#"
    const _: u32 = (5);
    "#,
    )
    .unwrap();
}

#[test]
fn test_array_index() {
    parse(
        r#"
    const _: u32 = test[1];
    "#,
    )
    .unwrap();
}

#[test]
fn test_member() {
    parse(
        r#"
    const _: u32 = test[5].west;
    const _: u32 = test.west;
    "#,
    )
    .unwrap();
}

#[test]
fn test_array_index_expr() {
    parse(
        r#"
    const _: u32 = test[1 + 1];
    "#,
    )
    .unwrap();
}

#[test]
fn test_negate() {
    parse(
        r#"
    const _: u32 = --1;
    "#,
    )
    .unwrap();
}

#[test]
fn test_not() {
    parse(
        r#"
    const _: u32 = !!false;
    "#,
    )
    .unwrap();
}

#[test]
fn test_bitnot() {
    parse(
        r#"
    const _: u32 = ~~1;
    "#,
    )
    .unwrap();
}

#[test]
fn test_bitnot_negate() {
    parse(
        r#"
    const _: u32 = ~-~-1;
    "#,
    )
    .unwrap();
}

#[test]
fn test_cast() {
    parse(
        r#"
    const _: u32 = 1 :> i32 :> u32 :> test;
    "#,
    )
    .unwrap();
}

#[test]
fn test_mul() {
    parse(
        r#"
    const _: u32 = 1 * 2 / 3 % 4;
    "#,
    )
    .unwrap();
}

#[test]
fn test_add() {
    parse(
        r#"
    const _: u32 = 1 + 3 - 2;
    "#,
    )
    .unwrap();
}

#[test]
fn test_shift() {
    parse(
        r#"
    const _: u32 = 1 << 5 >> 1 >>> 1;
    "#,
    )
    .unwrap();
}

#[test]
fn test_rel() {
    parse(
        r#"
    const _: u32 = 2 > 1 && 1 < 2 && 2 <= 2 && 2 >= 2;
    "#,
    )
    .unwrap();
}

#[test]
fn test_eq() {
    parse(
        r#"
    const _: u32 = 5 == 2 != false;
    "#,
    )
    .unwrap();
}

#[test]
fn test_bit_and() {
    parse(
        r#"
    const _: u32 = 5 & 2;
    "#,
    )
    .unwrap();
}

#[test]
fn test_bit_xor() {
    parse(
        r#"
    const _: u32 = 5 ^ 2;
    "#,
    )
    .unwrap();
}

#[test]
fn test_bit_or() {
    parse(
        r#"
    const _: u32 = 5 | 8;
    "#,
    )
    .unwrap();
}

#[test]
fn test_and() {
    parse(
        r#"
    const _: u32 = true && false;
    "#,
    )
    .unwrap();
}

#[test]
fn test_or() {
    parse(
        r#"
    const _: u32 = true || false;
    "#,
    )
    .unwrap();
}

#[test]
fn test_ternary() {
    parse(
        r#"
    const _: u32 = true ? false ? 3 : 2 : 1;
    "#,
    )
    .unwrap();
}
