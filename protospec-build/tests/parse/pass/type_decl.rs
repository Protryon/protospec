use crate::*;

#[test]
fn test_basic() {
    parse(
        r#"
    type test = u32;
    "#,
    )
    .unwrap();
}

#[test]
fn test_container() {
    parse(
        r#"
    type test = container {
        west: u32,
    };
    "#,
    )
    .unwrap();
}

#[test]
fn test_container_le() {
    parse(
        r#"
    type test = container {
        west: u32le,
    };
    "#,
    )
    .unwrap();
}

#[test]
fn test_container_empty() {
    parse(
        r#"
    type test = container {};
    "#,
    )
    .unwrap();
}

#[test]
fn test_container_no_trailing() {
    parse(
        r#"
    type test = container {
        west: u32
    };
    "#,
    )
    .unwrap();
}

#[test]
fn test_nested_container() {
    parse(
        r#"
    type test = container {
        west: container {
            east: u32,
        },
    };
    "#,
    )
    .unwrap();
}

#[test]
fn test_container_pad() {
    parse(
        r#"
    type test = container {
        x: u32,
        .pad: 5,
    };
    "#,
    )
    .unwrap();
}

#[test]
fn test_container_length_expand() {
    parse(
        r#"
    type test = container {
        west: u32,
    } [..];
    "#,
    )
    .unwrap();
}

#[test]
fn test_container_length() {
    parse(
        r#"
    type test = container [5] {
        west: u32,
    };
    "#,
    )
    .unwrap();
}

#[test]
fn test_container_array() {
    parse(
        r#"
    type test = container [5] {
        west: u8[4],
    };
    "#,
    )
    .unwrap();
}

#[test]
fn test_array_container_array() {
    parse(
        r#"
    type test = container [5] {
        west: u8[4],
    }[3];
    "#,
    )
    .unwrap();
}

#[test]
fn test_array() {
    parse(
        r#"
    type test = u8[4];
    "#,
    )
    .unwrap();
}

#[test]
fn test_array_2d() {
    parse(
        r#"
    type test = u8[4][3];
    "#,
    )
    .unwrap();
}

#[test]
fn test_float() {
    parse(
        r#"
    type test = f32;
    "#,
    )
    .unwrap();
}

#[test]
fn test_bool() {
    parse(
        r#"
    type test = bool;
    "#,
    )
    .unwrap();
}

#[test]
fn test_bitfield() {
    parse(
        r#"
    type test = bitfield i32 {
        test = 1,
        west,
        east,
        north,
        south,
    };
    "#,
    )
    .unwrap();
}

#[test]
fn test_enum() {
    parse(
        r#"
    type test = enum i32 {
        test = 1,
        west,
        east,
        north = 6,
        south,
    };
    "#,
    )
    .unwrap();
}

#[test]
fn test_enum_no_trailing() {
    parse(
        r#"
    type test = enum i32 {
        test = 1,
        west,
        east,
        north = 6,
        south
    };
    "#,
    )
    .unwrap();
}

#[test]
fn test_enum_array() {
    parse(
        r#"
    type test = enum i32 {
        test = 1,
        west,
        east,
        north = 6,
        south,
    }[10];
    "#,
    )
    .unwrap();
}

#[test]
fn test_enum_default() {
    parse(
        r#"
    type test = enum i32 {
        test = 1,
        west,
        east,
        north = 6,
        south,
        def = default,
    };
    "#,
    )
    .unwrap();
}

#[test]
fn test_conditional() {
    parse(
        r#"
    type test = container {
        len: u32,
        is_present: bool,
        data: u8[len] { is_present },
    };
    "#,
    )
    .unwrap();
}

#[test]
fn test_conditional_array() {
    parse(
        r#"
    type test = container {
        is_present: bool,
        data: u8[2][..] { is_present },
    };
    "#,
    )
    .unwrap();
}

#[test]
fn test_transform() {
    parse(
        r#"
    type test = container {
        data: u8[..] -> gzip,
    };
    "#,
    )
    .unwrap();
}

#[test]
fn test_transform_args() {
    parse(
        r#"
    type test = container {
        data: u8[..] -> base64("url"),
    };
    "#,
    )
    .unwrap();
}

#[test]
fn test_type_args() {
    parse(
        r#"
    type test(len: u32) = container {
        data: u8[len],
    };
    "#,
    )
    .unwrap();
}

#[test]
fn test_transform2() {
    parse(
        r#"
    type test = container {
        data: u8[..] -> gzip -> encrypt,
    };
    "#,
    )
    .unwrap();
}

#[test]
fn test_transform_conditional() {
    parse(
        r#"
    type test = container {
        len: u32,
        is_present: bool,
        data: u8[..] { is_present } -> gzip -> encrypt,
    };
    "#,
    )
    .unwrap();
}

#[test]
fn test_transform_conditional_transform() {
    parse(
        r#"
    type test = container {
        len: u32,
        is_present: bool,
        is_encrypted: bool,
        data: u8[6] { is_present } -> gzip -> encrypt {is_encrypted},
    };
    "#,
    )
    .unwrap();
}

#[test]
fn test_transform_conditional_transform_array() {
    parse(
        r#"
    type test = container {
        len: u32,
        is_present: bool,
        is_encrypted: bool,
        data: u8[6*len] { is_present } -> gzip -> encrypt {is_encrypted} -> base64,
    };
    "#,
    )
    .unwrap();
}
