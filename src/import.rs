use crate::asg::{ForeignTransformObj, ForeignTypeObj};
use crate::result::*;

pub trait ImportResolver {
    fn normalize_import(&self, import: &str) -> Result<String>;

    fn resolve_import(&self, import: &str) -> Result<Option<String>>;

    fn resolve_ffi_transform(&self, name: &str) -> Result<Option<ForeignTransformObj>>;

    fn resolve_ffi_type(&self, name: &str) -> Result<Option<ForeignTypeObj>>;
}

pub struct NullImportResolver;

impl ImportResolver for NullImportResolver {
    fn normalize_import(&self, import: &str) -> Result<String> {
        Ok(import.to_string())
    }

    fn resolve_import(&self, _import: &str) -> Result<Option<String>> {
        Err(protospec_err!("null import resolver"))
    }

    fn resolve_ffi_transform(&self, _transform: &str) -> Result<Option<ForeignTransformObj>> {
        Err(protospec_err!("null import resolver"))
    }

    fn resolve_ffi_type(&self, _type: &str) -> Result<Option<ForeignTypeObj>> {
        Err(protospec_err!("null import resolver"))
    }
}
