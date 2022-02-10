use crate::*;

#[test]
fn test_int() {
    load_asg(
        r#"
    const _: u32 = 5;
    "#,
    )
    .unwrap();
}

#[test]
fn test_up_int() {
    load_asg(
        r#"
    const _: u32 = 5u16;
    "#,
    )
    .unwrap();
}

#[test]
fn test_hex_int() {
    load_asg(
        r#"
    const _: u32 = 0x5FF;
    "#,
    )
    .unwrap();
}

#[test]
fn test_string() {
    load_asg(
        r#"
    const _: u8[..] = "test";
    "#,
    )
    .unwrap();
}

#[test]
fn test_string_escaped() {
    load_asg(
        r#"
    const _: u8[..] = "te\"st";
    "#,
    )
    .unwrap();
}

#[test]
fn test_ident() {
    load_asg(
        r#"
    const test: u32 = 2;
    const _: u32 = test;
    "#,
    )
    .unwrap();
}

#[test]
fn test_paren() {
    load_asg(
        r#"
    const _: u32 = (5);
    "#,
    )
    .unwrap();
}

#[test]
fn test_array_index() {
    load_asg(
        r#"
    const test: u8[..] = "test";
    const _: u8 = test[1];
    "#,
    )
    .unwrap();
}

#[test]
fn test_array_index_expr() {
    load_asg(
        r#"
    const test: u8[..] = "test";
    const _: u8 = test[1 + 1];
    const _2: u8 = test[1] + test[2];
    "#,
    )
    .unwrap();
}

#[test]
fn test_negate() {
    load_asg(
        r#"
    const _: i32 = --1;
    "#,
    )
    .unwrap();
}

#[test]
fn test_not() {
    load_asg(
        r#"
    const _: bool = !!false;
    "#,
    )
    .unwrap();
}

#[test]
fn test_bitnot() {
    load_asg(
        r#"
    const _: u32 = ~~1;
    "#,
    )
    .unwrap();
}

#[test]
fn test_bitnot_negate() {
    load_asg(
        r#"
    const _: i32 = ~-~-1;
    "#,
    )
    .unwrap();
}

#[test]
fn test_cast() {
    load_asg(
        r#"
    const _: u32 = 1i64 :> i32 :> u16;
    "#,
    )
    .unwrap();
}

#[test]
fn test_mul() {
    load_asg(
        r#"
    const _: u32 = 1 * 2 / 3 % 4;
    "#,
    )
    .unwrap();
}

#[test]
fn test_add() {
    load_asg(
        r#"
    const _: u32 = 1 + 3 - 2;
    "#,
    )
    .unwrap();
}

#[test]
fn test_shift() {
    load_asg(
        r#"
    const _: u32 = 1 << 5 >> 1 >>> 1;
    "#,
    )
    .unwrap();
}

#[test]
fn test_rel() {
    load_asg(
        r#"
    const _: bool = 2i32 > 1 && 1 < 2i32 && 2i32 <= 2 && 2 >= 2i32;
    "#,
    )
    .unwrap();
}

#[test]
fn test_eq() {
    load_asg(
        r#"
    const _: bool = 5i32 == 2 != false;
    "#,
    )
    .unwrap();
}

#[test]
fn test_bit_and() {
    load_asg(
        r#"
    const _: u32 = 5 & 2u32;
    "#,
    )
    .unwrap();
}

#[test]
fn test_bit_xor() {
    load_asg(
        r#"
    const _: u32 = 5u32 ^ 2;
    "#,
    )
    .unwrap();
}

#[test]
fn test_bit_or() {
    load_asg(
        r#"
    const _: u32 = 5 | 8u32;
    "#,
    )
    .unwrap();
}

#[test]
fn test_and() {
    load_asg(
        r#"
    const _: bool = true && false;
    "#,
    )
    .unwrap();
}

#[test]
fn test_or() {
    load_asg(
        r#"
    const _: bool = true || false;
    "#,
    )
    .unwrap();
}

#[test]
fn test_ternary() {
    load_asg(
        r#"
    const _: u32 = true ? false ? 3 : 2 : 1;
    "#,
    )
    .unwrap();
}
