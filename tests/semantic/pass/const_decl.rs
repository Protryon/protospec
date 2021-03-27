use crate::*;

#[test]
fn test_const() {
    load_asg(
        r#"
    const TEST: u32 = 5;
    "#,
    )
    .unwrap();
}

#[test]
fn test_const_array() {
    load_asg(
        r#"
    const TEST: u8[..] = "test";
    "#,
    )
    .unwrap();
}

#[test]
fn test_enum_const() {
    load_asg(
        r#"
    type color = enum i32 {
        red = 1,
        green,
        blue,
    };
    const TEST: color = 5i32;
    const TEST2: color = color.red;
    "#,
    )
    .unwrap();
}
