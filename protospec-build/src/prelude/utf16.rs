use super::*;

pub struct Utf16;

impl ForeignType for Utf16 {
    fn assignable_from(&self, type_: &Type) -> bool {
        match type_ {
            Type::Array(inner) => {
                Type::Scalar(ScalarType::U16.into()).assignable_from(&*inner.element)
            }
            _ => false,
        }
    }

    fn assignable_to(&self, type_: &Type) -> bool {
        self.assignable_from(type_)
    }

    fn type_ref(&self) -> TokenStream {
        quote! { String }
    }

    fn decoding_gen(
        &self,
        source: TokenStream,
        output_ref: TokenStream,
        arguments: Vec<TokenStream>,
        is_async: bool,
    ) -> TokenStream {
        let async_ = map_async(is_async);
        let len = arguments.first().expect("missing len argument");
        quote! {
            let #output_ref = {
                let t_count = #len as usize;
                let mut t: Vec<u16> = Vec::with_capacity(t_count);
                unsafe { t.set_len(t_count); }
                let t_borrow = &mut t[..];
                let t_borrow2 = unsafe {
                    let len = t_borrow.len() * 2;
                    let ptr = t.as_ptr() as *mut u8;
                    slice::from_raw_parts_mut(ptr, len)
                };
                #source.read_exact(&mut t_borrow2[..])#async_?;
                String::from_utf16(&t[..])?
            };
        }
    }

    fn encoding_gen(
        &self,
        target: TokenStream,
        field_ref: TokenStream,
        _arguments: Vec<TokenStream>,
        is_async: bool,
    ) -> TokenStream {
        let async_ = map_async(is_async);
        quote! {
            {
                for utf16 in #field_ref.encode_utf16() {
                    #target.write_all(&utf16.to_be_bytes()[..])#async_?;
                }
            }
        }
    }

    fn arguments(&self) -> Vec<TypeArgument> {
        vec![TypeArgument {
            name: "length".to_string(),
            type_: Type::Scalar(ScalarType::U64.into()),
            default_value: None,
            can_resolve_auto: true,
        }]
    }

    fn copyable(&self) -> bool {
        false
    }
}
