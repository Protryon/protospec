use super::*;

pub struct BitsFunction;

impl ForeignFunction for BitsFunction {
    fn arguments(&self) -> Vec<FFIArgument> {
        vec![
            FFIArgument {
                name: "input".to_string(),
                type_: Some(Type::Scalar(ScalarType::U64)),
                optional: false,
            },
        ]
    }

    fn return_type(&self) -> Type {
        Type::Scalar(ScalarType::U8)
    }

    fn call(&self, arguments: &[FFIArgumentValue]) -> TokenStream {
        let input = &arguments[0].value;
        return quote! {
            ((#input).count_ones() as u8)
        };
    }
}
