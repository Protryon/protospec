

#[macro_export]
macro_rules! include_spec {
    ($package: tt) => {
        include!(concat!(env!("OUT_DIR"), concat!("/", $package, ".rs")));
    };
}