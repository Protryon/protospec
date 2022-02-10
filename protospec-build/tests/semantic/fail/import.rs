use crate::*;

fn base_import() -> MockImportResolver {
    let imported = r#"

    import_ffi test_transform as transform;
    import_ffi test_type as type;

    type test_container = container {
        len: u32,
        buf: u8[len*2],
    };

    type test_container2 = container {
        data: test_type -> test_transform[2],
    };
    "#;
    let mut mocked_import = MockImportResolver(IndexMap::new());
    mocked_import
        .0
        .insert("test-import".to_string(), imported.to_string());
    mocked_import
        .0
        .insert("malformed".to_string(), ";".to_string());
    mocked_import
}

#[test]
fn test_import_missing() {
    let mocked_import = base_import();

    load_asg_with(
        r#"
    import test_missing from "test-import";

    type test_impl = test_missing;
    "#,
        mocked_import,
    )
    .err()
    .unwrap();
}

#[test]
fn test_import_file_missing() {
    let mocked_import = base_import();

    load_asg_with(
        r#"
    import test_container from "test-import_missing";

    type test_impl = test_container;
    "#,
        mocked_import,
    )
    .err()
    .unwrap();
}

#[test]
fn test_import_file_corrupt() {
    let mocked_import = base_import();

    load_asg_with(
        r#"
    import test_container from "malformed";

    type test_impl = test_container;
    "#,
        mocked_import,
    )
    .err()
    .unwrap();
}
