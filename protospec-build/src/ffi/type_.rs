use proc_macro2::TokenStream;

use crate::{asg::{Type, TypeArgument}, PartialType, ScalarType, PartialScalarType};


pub type ForeignTypeObj = Box<dyn ForeignType + 'static>;

/// An encodable and decodable foreign type object
/// Generally these are used to represent complexly encoded types,
/// where there isn't a notion of smaller internal types
/// 
/// Good examples:
/// Custom encoded integers (i.e. varints)
/// UTF8 strings
/// 
/// Bad examples:
/// GZIP encoded data
/// Encrypted data
pub trait ForeignType {
    /// Is this type assignable from this other type?
    /// i.e. let X: ThisType = OtherType;
    fn assignable_from(&self, type_: &Type) -> bool;

    /// Is this type assignable to this other type?
    /// i.e. let X: OtherType = ThisType;
    /// Generally identical to [`ForeignType::assignable_from`]
    fn assignable_to(&self, type_: &Type) -> bool;

    /// An internal modified form of [`ForeignType::assignable_from`] to provide some flexibility in type checking.
    fn assignable_from_partial(&self, type_: &PartialType) -> bool {
        match type_ {
            PartialType::Type(t) => self.assignable_from(t),
            PartialType::Any => true,
            PartialType::Scalar(PartialScalarType::Some(scalar)) |
            PartialType::Scalar(PartialScalarType::Defaults(scalar))
                => self.assignable_from(&Type::Scalar(*scalar)),
            _ => false,
        }
    }

    /// An internal modified form of [`ForeignType::assignable_to`] to provide some flexibility in type checking.
    fn assignable_to_partial(&self, type_: &PartialType) -> bool {
        match type_ {
            PartialType::Type(t) => self.assignable_to(t),
            PartialType::Any => true,
            PartialType::Scalar(PartialScalarType::Some(scalar)) |
            PartialType::Scalar(PartialScalarType::Defaults(scalar))
                => self.assignable_to(&Type::Scalar(*scalar)),
            _ => false,
        }
    }

    /// Emits this type as a Rust type
    fn type_ref(&self) -> TokenStream;

    /**
     *  output code should be a term expression that:
     *   1. the expression should read its input from an implicit identifier `reader` as a `&mut R` where R: Read
     *   2. can read an arbitrary number of bytes from `reader`
     *   3. returns a value of the foreign type
    */
    fn decoding_gen(
        &self,
        source: TokenStream,
        output_ref: TokenStream,
        arguments: Vec<TokenStream>,
        is_async: bool,
    ) -> TokenStream;

    /**
     * output code should be a single statement that:
     *  1. takes an expression `field_ref` as a reference to a value of the foreign type
     *  2. the statement should write its output to an implicit identifier `writer` as a `&mut W` where W: Write
    */
    fn encoding_gen(
        &self,
        target: TokenStream,
        field_ref: TokenStream,
        arguments: Vec<TokenStream>,
        is_async: bool,
    ) -> TokenStream;

    /// All arguments that can be passed to this type to describe characteristics (i.e. string length)
    /// All optional arguments must come at the end of the list of arguments.
    fn arguments(&self) -> Vec<TypeArgument>;

    /// If `Some`, this type can receive `auto` defined lengths during encoding
    fn can_receive_auto(&self) -> Option<ScalarType>;
}
