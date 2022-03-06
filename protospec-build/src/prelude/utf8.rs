use super::*;

pub struct Utf8;

impl ForeignType for Utf8 {
    fn assignable_from(&self, type_: &Type) -> bool {
        match type_ {
            Type::Array(inner) => {
                Type::Scalar(ScalarType::U8.into()).assignable_from(&*inner.element)
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
        if let Some(len) = arguments.first() {
            quote! {
                let #output_ref = {
                    let t_count = #len as usize;
                    let mut t: Vec<u8> = Vec::with_capacity(t_count);
                    unsafe { t.set_len(t_count); }
                    let t_borrow = &mut t[..];
                    let t_borrow2 = unsafe {
                        let len = t_borrow.len();
                        let ptr = t.as_ptr() as *mut u8;
                        slice::from_raw_parts_mut(ptr, len)
                    };
                    #source.read_exact(&mut t_borrow2[..])#async_?;
                    String::from_utf8(t)?
                };
            }
        } else {
            quote! {
                let #output_ref = {
                    let mut t: Vec<u8> = vec![];
                    #source.read_until(0u8, &mut t)#async_?;
                    if t.len() > 0 && t[t.len() - 1] == 0u8 {
                        t.truncate(t.len() - 1);
                    }
                    String::from_utf8(t)?
                };
            }
        }
    }

    fn encoding_gen(
        &self,
        target: TokenStream,
        field_ref: TokenStream,
        arguments: Vec<TokenStream>,
        is_async: bool,
    ) -> TokenStream {
        let async_ = map_async(is_async);
        if let Some(_) = arguments.first() {
            quote! {
                {
                    #target.write_all(#field_ref.as_bytes())#async_?;
                }
            }
        } else {
            quote! {
                {
                    #target.write_all(#field_ref.as_bytes())#async_?;
                    #target.write_all(&[0u8])#async_?;
                }
            }
        }
    }

    fn arguments(&self) -> Vec<TypeArgument> {
        vec![TypeArgument {
            name: "length".to_string(),
            type_: Type::Scalar(ScalarType::U64.into()),
            default_value: Some(u64::MAX.into()),
            can_resolve_auto: true,
        }]
    }

    fn copyable(&self) -> bool {
        false
    }
}
