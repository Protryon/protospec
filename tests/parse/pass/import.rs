use crate::*;

#[test]
fn test_import() {
    parse(
        r#"
    import test from "t";
    "#,
    )
    .unwrap();
}

#[test]
fn test_import_many() {
    parse(
        r#"
    import test, west from "t";
    "#,
    )
    .unwrap();
}

#[test]
fn test_import_many_trailing() {
    parse(
        r#"
    import test, west, from "t";
    "#,
    )
    .unwrap();
}
