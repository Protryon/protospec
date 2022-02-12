use crate::{emit_type_ref, PartialType};

use super::*;

pub struct VarInt {
    scalar_type: ScalarType,
    unsigned: Type,
}

impl VarInt {
    pub fn new(scalar_type: ScalarType) -> Self {
        VarInt {
            scalar_type,
            unsigned: Type::Scalar(match scalar_type {
                x if !x.is_signed() => x,
                ScalarType::I8 => ScalarType::U8,
                ScalarType::I16 => ScalarType::U16,
                ScalarType::I32 => ScalarType::U32,
                ScalarType::I64 => ScalarType::U64,
                ScalarType::I128 => ScalarType::U128,
                _ => unimplemented!(),
            }),
        }
    }
}

impl ForeignType for VarInt {
    fn assignable_from(&self, type_: &Type) -> bool {
        Type::Scalar(self.scalar_type).assignable_from(type_)
    }

    fn assignable_from_partial(&self, type_: &PartialType) -> bool {
        match type_ {
            PartialType::Type(t) => self.assignable_from(t),
            PartialType::Scalar(Some(scalar)) => self.assignable_from(&Type::Scalar(*scalar)),
            _ => false,
        }
    }

    fn assignable_to_partial(&self, type_: &PartialType) -> bool {
        match type_ {
            PartialType::Type(t) => self.assignable_to(t),
            PartialType::Any => true,
            PartialType::Scalar(Some(scalar)) => self.assignable_from(&Type::Scalar(*scalar)),
            PartialType::Scalar(None) => true,
            _ => false,
        }
    }

    fn assignable_to(&self, type_: &Type) -> bool {
        type_.assignable_from(&Type::Scalar(self.scalar_type))
    }

    fn type_ref(&self) -> TokenStream {
        emit_type_ref(&Type::Scalar(self.scalar_type))
    }

    fn decoding_gen(
        &self,
        source: TokenStream,
        output_ref: TokenStream,
        _arguments: Vec<TokenStream>,
        is_async: bool,
    ) -> TokenStream {
        let inner = self.type_ref();
        let end = (self.scalar_type.size() as f64 / 7.0).ceil() as usize;
        let inner_unsigned = emit_type_ref(&self.unsigned);
        let async_ = map_async(is_async);
        quote! {
            let #output_ref = {
                let mut i = 0usize;
                let mut buf = [0xffu8; 1];
                let mut output: #inner = 0;
                while (buf[0] & 128) == 128 {
                    #source.read_exact(&mut buf[..])#async_?;
                    output |= ((buf[0] as #inner_unsigned & 127) << (i * 7)) as #inner;
                    i += 1;
                    if i > #end {
                        break;
                    }
                }
                output
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
        let inner_unsigned = emit_type_ref(&self.unsigned);
        let async_ = map_async(is_async);
        quote! {
            {
                let mut value = #field_ref.clone() as #inner_unsigned;
                while (value & !0b1111111) != 0 {
                    #target.write_all(&[(value as u8 & 127) | 128])#async_?;
                    value >>= 7;
                }
                #target.write_all(&[value as u8])#async_?;
            }
        }
    }

    fn arguments(&self) -> Vec<TypeArgument> {
        vec![]
    }

    fn can_receive_auto(&self) -> Option<ScalarType> {
        Some(self.scalar_type)
    }
}
