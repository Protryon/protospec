use crate::*;

#[test]
fn test_const_bad_implicit() {
    load_asg(
        r#"
    const TEST: i32 = 5u32;
    "#,
    )
    .err()
    .unwrap();
}

#[test]
fn test_const_complex_type() {
    load_asg(
        r#"
    const TEST: enum u32 { test = 0, } = 5u32;
    "#,
    )
    .err()
    .unwrap();
}

#[test]
fn test_const_redefine() {
    load_asg(
        r#"
    const TEST: i32 = 5;
    const TEST: i32 = 10;
    "#,
    )
    .err()
    .unwrap();
}

#[test]
fn test_const_invalid_type() {
    load_asg(
        r#"
    const TEST: x = 5;
    "#,
    )
    .err()
    .unwrap();
}

#[test]
fn test_const_recursive() {
    load_asg(
        r#"
    const TEST: u32 = TEST;
    "#,
    )
    .err()
    .unwrap();
}

#[test]
fn test_const_indirect_recursive() {
    load_asg(
        r#"
    const TEST1: u32 = TEST2;
    const TEST2: u32 = TEST1;
    "#,
    )
    .err()
    .unwrap();
}
