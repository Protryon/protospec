use crate::*;

#[test]
fn test_import_ident_from() {
    parse(
        r#"
    import test from t;
    "#,
    )
    .err()
    .unwrap();
}

#[test]
fn test_import_missing_from() {
    parse(
        r#"
    import test "t";
    "#,
    )
    .err()
    .unwrap();
}

#[test]
fn test_import_missing_from_clause() {
    parse(
        r#"
    import test;
    "#,
    )
    .err()
    .unwrap();
}

#[test]
fn test_import_empty() {
    parse(
        r#"
    import from "t";
    "#,
    )
    .err()
    .unwrap();
}
