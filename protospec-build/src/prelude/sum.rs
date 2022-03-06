use super::*;

pub struct SumFunction;

impl ForeignFunction for SumFunction {
    fn arguments(&self) -> Vec<FFIArgument> {
        vec![FFIArgument {
            name: "input".to_string(),
            type_: None,
            optional: false,
        }]
    }

    fn return_type(&self) -> Type {
        Type::Scalar(ScalarType::U64.into())
    }

    fn call(&self, arguments: &[FFIArgumentValue]) -> TokenStream {
        let type_ = &arguments[0].type_;
        match type_ {
            Type::Array(inner) => match &*inner.element {
                Type::Scalar(_) => (),
                _ => panic!("invalid array interior type for sum"),
            },
            _ => panic!("invalid type for sum, expected array"),
        }
        let input = &arguments[0].value;
        return quote! {
            (#input.iter().copied().map(|x| x as u64).sum::<u64>() as u64)
        };
    }
}
