
fn main() {
    protospec_build::compile_spec("nbt", include_str!("./spec/nbt.pspec"), &protospec_build::Options {
        include_async: true,
        ..Default::default()
    }).expect("failed to build nbt.pspec");
}