use super::*;
pub struct PreludeImportResolver<T: ImportResolver + 'static>(pub T);

impl<T: ImportResolver + 'static> ImportResolver for PreludeImportResolver<T> {
    fn normalize_import(&self, import: &str) -> Result<String> {
        self.0.normalize_import(import)
    }

    fn resolve_import(&self, import: &str) -> Result<Option<String>> {
        self.0.resolve_import(import)
    }

    fn resolve_ffi_transform(&self, transform: &str) -> Result<Option<ForeignTransformObj>> {
        Ok(match transform {
            "base64" => Some(Box::new(Base64Transform)),
            "gzip" => Some(Box::new(GzipTransform)),
            x => self.0.resolve_ffi_transform(x)?,
        })
    }

    fn resolve_ffi_type(&self, import: &str) -> Result<Option<ForeignTypeObj>> {
        Ok(match import {
            "v8" => Some(Box::new(VarInt::new(ScalarType::I8))),
            "v16" => Some(Box::new(VarInt::new(ScalarType::I16))),
            "v32" => Some(Box::new(VarInt::new(ScalarType::I32))),
            "v64" => Some(Box::new(VarInt::new(ScalarType::I64))),
            "v128" => Some(Box::new(VarInt::new(ScalarType::I128))),
            x => self.0.resolve_ffi_type(x)?,
        })
    }
}
