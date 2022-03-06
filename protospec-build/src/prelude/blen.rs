use super::*;

/// a special function used for generating encoders that returns the byte length of a future field by encoding it ahead of time
pub struct BLenFunction;

impl ForeignFunction for BLenFunction {
    fn arguments(&self) -> Vec<FFIArgument> {
        vec![FFIArgument {
            name: "target".to_string(),
            type_: None,
            optional: false,
        }]
    }

    fn return_type(&self) -> Type {
        Type::Scalar(ScalarType::U64)
    }

    fn call(&self, _arguments: &[FFIArgumentValue]) -> TokenStream {
        unimplemented!()
    }
}
