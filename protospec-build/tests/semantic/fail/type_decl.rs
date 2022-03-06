use crate::*;

#[test]
fn test_type_redefine() {
    load_asg(
        r#"
    type test = u32;
    type test = u64;
    "#,
    )
    .err()
    .unwrap();
}

#[test]
fn test_type_redefine2() {
    load_asg(
        r#"
    type test = u32;
    type test = u32;
    "#,
    )
    .err()
    .unwrap();
}

#[test]
fn test_container_redefine() {
    load_asg(
        r#"
    type test = container {
        west: u32,
        east: u32,
        west: u32,
    };
    "#,
    )
    .err()
    .unwrap();
}

#[test]
fn test_enum() {
    load_asg(
        r#"
    type test = enum i32 {
        test = 1u32,
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
fn test_enum_default_early() {
    load_asg(
        r#"
    type test = enum i32 {
        test = 1,
        def = default,
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
fn test_conditional_bad() {
    load_asg(
        r#"
    type test = container {
        len: u32,
        data: u8[len] { is_present },
    };
    "#,
    )
    .err()
    .unwrap();
}

#[test]
fn test_conditional_inverted() {
    load_asg(
        r#"
    type test = container {
        len: u32,
        data: u8[len] { is_present },
        is_present: bool,
    };
    "#,
    )
    .err()
    .unwrap();
}
