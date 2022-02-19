use super::*;

pub struct PadFunction;

impl ForeignFunction for PadFunction {
    fn arguments(&self) -> Vec<FFIArgument> {
        vec![
            FFIArgument {
                name: "pad".to_string(),
                type_: Some(Type::Scalar(ScalarType::U64)),
                optional: false,
            },
            FFIArgument {
                name: "base".to_string(),
                type_: Some(Type::Scalar(ScalarType::U64)),
                optional: false,
            }
        ]
    }

    fn return_type(&self) -> Type {
        Type::Scalar(ScalarType::U64)
    }

    fn call(&self, arguments: &[FFIArgumentValue]) -> TokenStream {
        let pad = &arguments[0].value;
        let base = &arguments[1].value;
        return quote! {
            if #base % #pad == 0 {
                #base
            } else {
                #base + (#pad - (#base % #pad))
            }
        };
    }
}
