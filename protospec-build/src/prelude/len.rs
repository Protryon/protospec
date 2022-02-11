use super::*;

pub struct LenFunction;

impl LenFunction {
    fn len(&self, type_: &Type, value: &TokenStream) -> TokenStream {
        match &type_ {
            Type::Container(_) => panic!("cannot call len on container"),
            Type::Enum(e) => {
                let size = e.rep.size();
                quote! {
                    #size
                }
            }
            Type::F32 => quote! { 4u64 },
            Type::F64 => quote! { 8u64 },
            Type::Bool => quote! { 1u64 },
            Type::Scalar(s) => {
                let size = s.size();
                quote! {
                    #size
                }
            }
            Type::Array(_) | Type::Foreign(_) => {
                quote! {
                    (#value).len() as u64
                }
            }
            Type::Ref(type_call) => self.len(&*type_call.target.type_.borrow(), value),
        }
    }
}

impl ForeignFunction for LenFunction {
    fn arguments(&self) -> Vec<FFIArgument> {
        vec![FFIArgument {
            name: "list".to_string(),
            type_: None,
            optional: false,
        }]
    }

    fn return_type(&self) -> Type {
        Type::Scalar(ScalarType::U64)
    }

    fn call(&self, arguments: &[FFIArgumentValue]) -> TokenStream {
        self.len(&arguments[0].type_, &arguments[0].value)
    }
}
