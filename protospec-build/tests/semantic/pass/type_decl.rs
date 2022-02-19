use crate::*;

#[test]
fn test_basic() {
    load_asg(
        r#"
    type test = u32;
    "#,
    )
    .unwrap();
}

#[test]
fn test_container() {
    load_asg(
        r#"
    type test = container {
        west: u32,
    };
    "#,
    )
    .unwrap();
}

#[test]
fn test_nested_container() {
    load_asg(
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
fn test_nested_container_scoping() {
    load_asg(
        r#"
    type test = container {
        east: u8[..],
        west: container {
            east: u32,
            west: u8[east]
        },
    };
    "#,
    )
    .unwrap();
}

#[test]
fn test_array_length_expand() {
    load_asg(
        r#"
    type testcontainer = container {
        west: u32,
    };
    type test = testcontainer[..];
    "#,
    )
    .unwrap();
}

#[test]
fn test_container_length() {
    load_asg(
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
    load_asg(
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
    load_asg(
        r#"
    type testcontainer = container [5] {
        west: u8[4],
    };
    type test = testcontainer[3];
    "#,
    )
    .unwrap();
}

#[test]
fn test_array() {
    load_asg(
        r#"
    type test = u8[4];
    "#,
    )
    .unwrap();
}

#[test]
fn test_array_2d() {
    load_asg(
        r#"
    type test = u8[4][3];
    "#,
    )
    .unwrap();
}

#[test]
fn test_float() {
    load_asg(
        r#"
    type test = f32;
    "#,
    )
    .unwrap();
}

#[test]
fn test_bool() {
    load_asg(
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
    load_asg(
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
fn test_enum_array() {
    load_asg(
        r#"
    type testenum = enum i32 {
        test = 1,
        west,
        east,
        north = 6,
        south,
    };
    type test = testenum[10];
    "#,
    )
    .unwrap();
}

#[test]
fn test_conditional() {
    load_asg(
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
    load_asg(
        r#"
    type test = container {
        is_present: bool,
        data: u8[2] { is_present } [..],
    };
    "#,
    )
    .unwrap();
}

#[test]
fn test_transform() {
    load_asg(
        r#"
    import_ffi test_transform as transform;

    type test = container {
        data: u8[..] -> test_transform,
    };
    "#,
    )
    .unwrap();
}

#[test]
fn test_transform2() {
    load_asg(
        r#"
    import_ffi test_transform as transform;
    type test = container {
        data: u8[..] -> test_transform -> test_transform,
    };
    "#,
    )
    .unwrap();
}

#[test]
fn test_transform_conditional() {
    load_asg(
        r#"
    import_ffi test_transform as transform;

    type test = container {
        len: u32,
        is_present: bool,
        data: u8[..] { is_present } -> test_transform -> test_transform,
    };
    "#,
    )
    .unwrap();
}

#[test]
fn test_transform_conditional_array() {
    load_asg(
        r#"
    import_ffi test_transform as transform;

    type test = container {
        len: u32,
        is_present: bool,
        data: u8[6] { is_present } -> test_transform -> test_transform[..],
    };
    "#,
    )
    .unwrap();
}

#[test]
fn test_transform_conditional_transform() {
    load_asg(
        r#"
    import_ffi test_transform as transform;

    type test = container {
        len: u32,
        is_present: bool,
        is_encrypted: bool,
        data: u8[6] { is_present } -> test_transform -> test_transform {is_encrypted},
    };
    "#,
    )
    .unwrap();
}

#[test]
fn test_transform_conditional_transform_array() {
    load_asg(r#"
    import_ffi test_transform as transform;

    type test = container {
        len: u32,
        is_present: bool,
        is_encrypted: bool,
        data: u8[6] { is_present } -> test_transform -> test_transform {is_encrypted}[len] -> test_transform,
    };
    "#).unwrap();
}
