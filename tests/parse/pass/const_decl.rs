use crate::*;

#[test]
fn test_const() {
    parse(
        r#"
    const TEST: u32 = 5;
    "#,
    )
    .unwrap();
}

#[test]
fn test_enum_const() {
    parse(
        r#"
    type color = enum i32 {
        red = 1,
        green,
        blue,
    };
    const TEST: color = 5;
    "#,
    )
    .unwrap();
}
